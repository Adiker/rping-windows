use std::mem;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::format::format_time;
use crate::icmp_status::icmp_status_to_str;
use crate::stats::PingStats;
use crate::win_icmp::{EchoReply, IcmpHandle, last_error, send_echo};

pub struct PingSession {
    pub stats: PingStats,
    pub duration: Duration,
}

pub fn run_ping(target_ip: Ipv4Addr, config: &Config) -> Result<PingSession, String> {
    let icmp_handle = IcmpHandle::open().map_err(|err| format!("IcmpCreateFile failed: {}", err))?;

    // IPAddr dla IcmpSendEcho powinien mieć taki sam układ jak IN_ADDR::S_addr
    // (jak z inet_addr), czyli "native-endian" dla aktualnej architektury.
    // Użycie from_be_bytes powodowało odwrócenie adresu na little-endian
    // i błędne statusy odpowiedzi (np. 11013 nawet dla 127.0.0.1).
    let dest_addr = u32::from_ne_bytes(target_ip.octets());

    let payload = vec![b'P'; config.payload_size];
    let reply_buf_size = mem::size_of::<EchoReply>() + config.payload_size + 8 + 256;
    let mut reply_buf = vec![0u8; reply_buf_size];

    let mut stats = PingStats::new();
    let total_iterations = if config.infinite {
        u32::MAX
    } else {
        config.count
    };

    let session_start = Instant::now();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| format!("Error setting Ctrl+C handler: {}", e))?;

    for seq in 0..total_iterations {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        reply_buf.iter_mut().for_each(|b| *b = 0);
        let start = Instant::now();

        let ret = send_echo(
            &icmp_handle,
            dest_addr,
            &payload,
            &mut reply_buf,
            config.timeout_ms,
        );

        let elapsed_ms = start.elapsed().as_millis() as u32;

        if ret == 0 {
            let err = last_error();
            if err == 11010 {
                println!("Request timeout for icmp_seq {}", seq);
            } else {
                println!("Request failed for icmp_seq {} (error={})", seq, err);
            }
            stats.record_failure();
        } else {
            let reply = unsafe { &*(reply_buf.as_ptr() as *const EchoReply) };
            let status = reply.Status;

            if status == 0 {
                let ttl = reply.Options.Ttl;
                let rtt = if reply.RoundTripTime == 0 {
                    elapsed_ms
                } else {
                    reply.RoundTripTime
                };
                println!(
                    "Reply from {}: bytes={} time={} TTL={}",
                    target_ip,
                    config.payload_size,
                    format_time(rtt),
                    ttl
                );
                stats.record_success(rtt);
            } else {
                println!(
                    "Reply from {}: {} (status={})",
                    target_ip,
                    icmp_status_to_str(status),
                    status
                );
                stats.record_failure();
            }
        }

        if seq + 1 < total_iterations && running.load(Ordering::SeqCst) {
            let elapsed = start.elapsed();
            let target_interval = Duration::from_millis(config.interval_ms);
            if elapsed < target_interval {
                std::thread::sleep(target_interval - elapsed);
            }
        }
    }

    Ok(PingSession {
        stats,
        duration: session_start.elapsed(),
    })
}
