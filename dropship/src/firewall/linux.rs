//! Native Linux firewall backend. The GUI stays unprivileged; a small installed
//! polkit helper owns only this user's Dropship nftables table / iptables chains.
use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::api::KnownServer;

const HELPER: &str = "/usr/libexec/dropship-firewall";

pub fn acquire_instance_lock() -> io::Result<File> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").ok_or_else(|| {
        io::Error::other("XDG_RUNTIME_DIR is missing; start Dropship in a desktop session")
    })?;
    let path = PathBuf::from(runtime).join("dropship.lock");
    let file = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    file.try_lock()
        .map_err(|e| io::Error::other(format!("Dropship is already running: {e}")))?;
    Ok(file)
}

struct Helper {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl Helper {
    fn start() -> io::Result<Self> {
        if !std::path::Path::new(HELPER).is_file() {
            return Err(io::Error::other(
                "Linux firewall helper is not installed. Run packaging/linux/install.sh first.",
            ));
        }
        let mut child = Command::new("/usr/bin/pkexec")
            .arg(HELPER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        Ok(Self {
            child,
            input,
            output,
        })
    }

    fn apply(&mut self, networks: &[String], persistent: bool) -> io::Result<()> {
        let request = serde_json::json!({ "networks": networks, "persistent": persistent });
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{request}")?;
        input.flush()?;
        let mut response = String::new();
        if self.output.read_line(&mut response)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Firewall authorization was cancelled or the helper exited. Try again.",
            ));
        }
        let response: serde_json::Value = serde_json::from_str(&response)?;
        if response["ok"] != true {
            return Err(io::Error::other(
                response["error"]
                    .as_str()
                    .unwrap_or("firewall helper failed")
                    .to_owned(),
            ));
        }
        if let Some(backend) = response["backend"].as_str() {
            log::debug!("Linux firewall backend: {backend}");
        }
        Ok(())
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        // EOF lets the root helper remove dynamic rules, even if the GUI crashes.
        drop(self.input.take());
        let _ = self.child.wait();
    }
}

pub struct Connection {
    persistent: bool,
    helper: Option<Helper>,
    last_applied: Option<Vec<String>>,
}

impl Connection {
    pub fn new(persistent: bool) -> io::Result<Self> {
        Ok(Self {
            persistent,
            helper: None,
            last_applied: None,
        })
    }

    fn apply(&mut self, networks: Vec<String>) -> io::Result<()> {
        if self
            .helper
            .as_mut()
            .is_some_and(|helper| !matches!(helper.child.try_wait(), Ok(None)))
        {
            self.helper = None;
            self.last_applied = None;
        }
        if self.last_applied.as_ref() == Some(&networks) && self.helper.is_some() {
            return Ok(());
        }
        // No authorization dialog on a fresh installation with nothing to block.
        // Saved state is written by the helper, never trusted as firewall commands.
        if networks.is_empty() && self.helper.is_none() {
            let uid = Command::new("/usr/bin/id").arg("-u").output()?;
            let uid = String::from_utf8_lossy(&uid.stdout)
                .trim()
                .parse::<u32>()
                .map_err(io::Error::other)?;
            if !PathBuf::from(format!("/var/lib/dropship/{uid}.json")).exists()
                && !PathBuf::from(format!("/var/lib/dropship/{uid}.runtime.json")).exists()
            {
                self.last_applied = Some(networks);
                return Ok(());
            }
        }
        if self.helper.is_none() {
            self.helper = Some(Helper::start()?);
        }
        let result = self
            .helper
            .as_mut()
            .unwrap()
            .apply(&networks, self.persistent);
        if result.is_err() {
            // A rejected transaction keeps the last working rules. Keep a live
            // dynamic helper too, or dropping it would clear those rules on EOF.
            if result.as_ref().is_err_and(|error| {
                matches!(
                    error.kind(),
                    io::ErrorKind::BrokenPipe | io::ErrorKind::UnexpectedEof
                )
            }) {
                self.helper = None;
            }
            self.last_applied = None;
        } else {
            self.last_applied = Some(networks);
            log::info!(
                "Linux firewall updated: {} networks, {}",
                self.last_applied.as_ref().unwrap().len(),
                if self.persistent {
                    "persistent"
                } else {
                    "while Dropship is open"
                }
            );
        }
        result
    }
}

pub fn apply_blocked_ips(
    connection: &mut Connection,
    blocked_servers: &HashSet<KnownServer>,
    already_known_paths: &HashSet<PathBuf>,
) -> io::Result<HashSet<PathBuf>> {
    let mut networks = BTreeSet::new();
    for server in blocked_servers {
        for network in server.block.split(',') {
            let network: ipnet::IpNet = network
                .trim()
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            if network.prefix_len() == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "refusing a default-route block",
                ));
            }
            networks.insert(network.trunc().to_string());
        }
    }
    connection.apply(networks.into_iter().collect())?;
    // Linux filters by UID, not executable path. Paths are informational here.
    Ok(already_known_paths
        .iter()
        .filter(|p| p.is_file())
        .cloned()
        .collect())
}
