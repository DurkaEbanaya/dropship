use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    str::FromStr,
    time::{Duration, Instant},
};
#[cfg(target_os = "windows")]
use tokio::time;

#[cfg(target_os = "windows")]
use rand::random;
#[cfg(target_os = "windows")]
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingSequence};

#[cfg(target_os = "windows")]
pub const PING_TIMEOUT: time::Duration = time::Duration::from_secs(2);
/// Delay after a completed measurement before trying again in continuous mode.
pub const REFRESH_DELAY: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct Measurements {
    results: HashMap<String, Result<f32, String>>,
    in_flight: HashSet<String>,
    completed_at: HashMap<String, Instant>,
}

impl Measurements {
    pub fn get(&self, ip: &str) -> Option<&Result<f32, String>> {
        self.results.get(ip)
    }

    pub fn is_measuring(&self, ip: &str) -> bool {
        self.in_flight.contains(ip)
    }

    pub fn start(&mut self, ip: &str) -> bool {
        self.in_flight.insert(ip.to_owned())
    }

    pub fn finish(&mut self, ip: String, result: Result<f32, String>, now: Instant) {
        self.in_flight.remove(&ip);
        self.completed_at.insert(ip.clone(), now);
        self.results.insert(ip, result);
    }

    pub fn due(&self, ip: &str, continuous: bool, now: Instant) -> bool {
        !self.is_measuring(ip)
            && self.completed_at.get(ip).is_none_or(|last| {
                continuous && now.saturating_duration_since(*last) >= REFRESH_DELAY
            })
    }

    pub fn retain_servers(&mut self, ips: &HashSet<String>) {
        self.results.retain(|ip, _| ips.contains(ip));
        self.completed_at.retain(|ip, _| ips.contains(ip));
        // Keep outstanding requests tracked until their replies arrive.
    }
}

#[cfg(target_os = "windows")]
const PING_INTERVAL: time::Duration = time::Duration::from_millis(900);

#[cfg(target_os = "windows")]
pub async fn ping_ip(ip: &String) -> Result<f32, String> {
    let ip = IpAddr::from_str(&ip).map_err(|e| e.to_string())?;

    let client = match ip {
        IpAddr::V4(_) => Client::new(&Config::default()).map_err(|e| e.to_string())?,
        IpAddr::V6(_) => {
            Client::new(&Config::builder().kind(ICMP::V6).build()).map_err(|e| e.to_string())?
        }
    };

    let payload = [0; 56];
    let mut pinger = client
        .pinger(ip, surge_ping::PingIdentifier(random::<u16>()))
        .await;

    pinger.timeout(PING_TIMEOUT);

    let mut interval = time::interval(PING_INTERVAL);

    let n_pings = 4;

    // REVIEW can maybe calculate n of hops here with 128 - ttl

    let mut ping = 0.;

    for idx in 0..n_pings {
        interval.tick().await;
        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(_packet), dur)) => {
                // println!(
                //     "No.{}: {} bytes from {}: icmp_seq={} ttl={:?} time={:0.2?}",
                //     idx,
                //     packet.get_size(),
                //     packet.get_source(),
                //     packet.get_sequence(),
                //     packet.get_ttl(),
                //     dur
                // )
                ping += dur.as_secs_f32() * 1000.;
                // ping += dur.as_millis_f32();
            }
            Ok((IcmpPacket::V6(_packet), dur)) => {
                // println!(
                //     "No.{}: {} bytes from {}: icmp_seq={} hlim={} time={:0.2?}",
                //     idx,
                //     packet.get_size(),
                //     packet.get_source(),
                //     packet.get_sequence(),
                //     packet.get_max_hop_limit(),
                //     dur
                // )
                ping += dur.as_secs_f32() * 1000.
                // ping += dur.as_millis_f32();
            }
            Err(e) => {
                let e = match e {
                    surge_ping::SurgeError::Timeout { .. } => format!("<{}> pinging failed", ip),
                    _ => e.to_string(),
                };

                // log::warn!("{}", e);

                return Err(e);
            }
        };
    }

    ping /= n_pings as f32;
    // println!("{} {:.2}ms", ip, ping);

    Ok(ping)
}

#[cfg(target_os = "linux")]
pub async fn ping_ip(ip: &String) -> Result<f32, String> {
    // openSUSE's iputils has the required ICMP capability; the GUI needs none.
    let ip = IpAddr::from_str(ip).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let output = std::process::Command::new("/usr/bin/ping")
            .env("LC_ALL", "C")
            .args([
                "-n", "-q", "-c", "4", "-i", "0.9", "-W", "2", "-w", "8", "--",
            ])
            .arg(ip.to_string())
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!("<{ip}> pinging failed"));
        }
        parse_ping_average(&String::from_utf8_lossy(&output.stdout))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "linux")]
fn parse_ping_average(output: &str) -> Result<f32, String> {
    output
        .lines()
        .find_map(|line| {
            let (_, values) = line.split_once(" = ")?;
            if !line.contains("min/avg/max") {
                return None;
            }
            values.split('/').nth(1)?.parse::<f32>().ok()
        })
        .ok_or_else(|| "could not parse ping average".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refreshes_are_deduplicated_and_keep_the_previous_result() {
        let mut pings = Measurements::default();
        let now = Instant::now();
        assert!(pings.due("127.0.0.1", false, now));
        assert!(pings.start("127.0.0.1"));
        assert!(!pings.start("127.0.0.1"));
        assert!(!pings.due("127.0.0.1", true, now + REFRESH_DELAY));
        pings.finish("127.0.0.1".into(), Ok(42.0), now);
        assert!(!pings.due("127.0.0.1", false, now + REFRESH_DELAY));
        assert!(!pings.due(
            "127.0.0.1",
            true,
            now + REFRESH_DELAY - Duration::from_millis(1)
        ));
        assert!(pings.due("127.0.0.1", true, now + REFRESH_DELAY));
        assert!(pings.start("127.0.0.1"));
        assert_eq!(pings.get("127.0.0.1"), Some(&Ok(42.0)));
    }

    #[test]
    fn errors_are_retried_only_in_continuous_mode_and_removed_servers_are_pruned() {
        let mut pings = Measurements::default();
        let now = Instant::now();
        pings.start("192.0.2.1");
        pings.finish("192.0.2.1".into(), Err("timeout".into()), now);
        assert!(!pings.due("192.0.2.1", false, now + REFRESH_DELAY));
        assert!(pings.due("192.0.2.1", true, now + REFRESH_DELAY));
        pings.start("192.0.2.1");
        pings.retain_servers(&HashSet::new());
        assert!(pings.get("192.0.2.1").is_none());
        assert!(!pings.due("192.0.2.1", true, now + REFRESH_DELAY));
        pings.finish("192.0.2.1".into(), Ok(12.0), now);
        pings.retain_servers(&HashSet::new());
        assert!(pings.due("192.0.2.1", false, now));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn iputils_average_and_timeout() {
        assert_eq!(
            super::parse_ping_average("rtt min/avg/max/mdev = 1.0/2.125/3.0/0.4 ms").unwrap(),
            2.125
        );
        assert!(super::parse_ping_average("4 packets transmitted, 0 received").is_err());
    }
}
