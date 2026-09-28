use std::{net::IpAddr, str::FromStr};
use tokio::time;

#[cfg(target_os = "windows")]
use rand::random;
#[cfg(target_os = "windows")]
use surge_ping::{Client, Config, ICMP, IcmpPacket, PingSequence};

pub const PING_TIMEOUT: time::Duration = time::Duration::from_secs(2);
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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn iputils_average_and_timeout() {
        assert_eq!(
            super::parse_ping_average("rtt min/avg/max/mdev = 1.0/2.125/3.0/0.4 ms").unwrap(),
            2.125
        );
        assert!(super::parse_ping_average("4 packets transmitted, 0 received").is_err());
    }
}
