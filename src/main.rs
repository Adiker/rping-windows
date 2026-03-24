use std::env;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY,
};
use windows_sys::Win32::Networking::WinSock::{WSACleanup, WSAStartup, WSADATA};

fn resolve_host(host: &str) -> Option<Ipv4Addr> {
    let addr_str = format!("{}:0", host);
    addr_str.to_socket_addrs().ok()?.find_map(|addr| {
        if let IpAddr::V4(ip) = addr.ip() {
            Some(ip)
        } else {
            None
        }
    })
}

fn format_time(ms: u32) -> String {
    if ms == 0 {
        "<1ms".to_string()
    } else {
        format!("{}ms", ms)
    }
}

fn format_duration(duration: Duration) -> String {
    let total_ms = duration.as_millis() as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let millis = total_ms % 1_000;

    if hours > 0 {
        format!("{}h {}m {}.{:03}s", hours, minutes, seconds, millis)
    } else if minutes > 0 {
        format!("{}m {}.{:03}s", minutes, seconds, millis)
    } else {
        format!("{}.{:03}s", seconds, millis)
    }
}

struct PingStats {
    sent: u32,
    received: u32,
    times: Vec<u32>,
}

impl PingStats {
    fn new() -> Self {
        PingStats { sent: 0, received: 0, times: Vec::new() }
    }

    fn record_success(&mut self, ms: u32) {
        self.sent += 1;
        self.received += 1;
        self.times.push(ms);
    }

    fn record_failure(&mut self) {
        self.sent += 1;
    }

    fn print_summary(&self) {
        let lost = self.sent - self.received;
        let loss_pct = if self.sent > 0 {
            (lost as f64 / self.sent as f64) * 100.0
        } else {
            0.0
        };

        println!("\nPing statistics:");
        println!(
            "    Packets: Sent = {}, Received = {}, Lost = {} ({:.0}% loss)",
            self.sent, self.received, lost, loss_pct
        );

        if !self.times.is_empty() {
            let min = *self.times.iter().min().unwrap();
            let max = *self.times.iter().max().unwrap();
            let avg = self.times.iter().sum::<u32>() / self.times.len() as u32;

            println!("Approximate round trip times in milli-seconds:");
            println!(
                "    Minimum = {}, Maximum = {}, Average = {}",
                format_time(min),
                format_time(max),
                format_time(avg)
            );
        }
    }
}

fn print_usage(program: &str) {
    eprintln!(
        "Usage: {} [-n count] [-w timeout] [-l size] [-i interval] [-S] [-t] <host>",
        program
    );
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n count    Number of echo requests (default: 4)");
    eprintln!("  -w timeout  Timeout in milliseconds (default: 4000)");
    eprintln!("  -l size     Payload size in bytes (default: 32)");
    eprintln!("  -i interval Interval between pings in milliseconds (default: 1000)");
    eprintln!("  -S          Show extra session summary (total duration and packet rate)");
    eprintln!("  -t          Ping indefinitely (Ctrl+C to stop)");
}

fn icmp_status_to_str(status: u32) -> &'static str {
    match status {
        11001 => "Destination Net Unreachable",
        11002 => "Destination Host Unreachable",
        11003 => "Destination Protocol Unreachable",
        11004 => "Destination Port Unreachable",
        11005 => "No Resources",
        11006 => "Bad Option",
        11007 => "Hardware Error",
        11008 => "Packet Too Big",
        11009 => "Request Timed Out",
        11010 => "Bad Request",
        11011 => "Bad Route",
        11012 => "TTL Expired Transit",
        11013 => "TTL Expired Reassembly",
        11014 => "Parameter Problem",
        11015 => "Source Quench",
        11016 => "Option Too Big",
        11017 => "Bad Destination",
        _ => "General Failure",
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    if args.len() < 2 {
        print_usage(program);
        std::process::exit(1);
    }

    let mut count: u32 = 4;
    let mut timeout_ms: u32 = 4000;
    let mut payload_size: usize = 32;
    let mut interval_ms: u64 = 1000;
    let mut show_session_summary = false;
    let mut infinite = false;
    let mut host = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                count = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(4);
            }
            "-w" => {
                i += 1;
                timeout_ms = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(4000);
            }
            "-l" => {
                i += 1;
                payload_size = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(32);
                payload_size = payload_size.min(65500);
            }
            "-i" => {
                i += 1;
                interval_ms = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1000);
                interval_ms = interval_ms.max(1);
            }
            "-t" => {
                infinite = true;
            }
            "-S" | "--session-summary" => {
                show_session_summary = true;
            }
            arg if !arg.starts_with('-') => {
                host = arg.to_string();
            }
            _ => {}
        }
        i += 1;
    }

    if host.is_empty() {
        eprintln!("Error: No host specified.");
        print_usage(program);
        std::process::exit(1);
    }

    unsafe {
        let mut wsa_data: WSADATA = mem::zeroed();
        let result = WSAStartup(0x0202, &mut wsa_data);
        if result != 0 {
            eprintln!("WSAStartup failed: {}", result);
            std::process::exit(1);
        }
    }

    let target_ip = match resolve_host(&host) {
        Some(ip) => ip,
        None => {
            eprintln!("Ping: cannot resolve '{}': Name or service not known.", host);
            unsafe { WSACleanup() };
            std::process::exit(1);
        }
    };

    let display_host = if host == target_ip.to_string() {
        host.clone()
    } else {
        format!("{} [{}]", host, target_ip)
    };

    println!("\nPinging {} with {} bytes of data:", display_host, payload_size);

    let icmp_handle = unsafe { IcmpCreateFile() };
    if icmp_handle == -1 {
        eprintln!("IcmpCreateFile failed: {}", unsafe { GetLastError() });
        unsafe { WSACleanup() };
        std::process::exit(1);
    }

    // IPAddr dla IcmpSendEcho powinien mieć taki sam układ jak IN_ADDR::S_addr
    // (jak z inet_addr), czyli "native-endian" dla aktualnej architektury.
    // Użycie from_be_bytes powodowało odwrócenie adresu na little-endian
    // i błędne statusy odpowiedzi (np. 11013 nawet dla 127.0.0.1).
    let dest_addr = u32::from_ne_bytes(target_ip.octets());

    let payload: Vec<u8> = vec![b'P'; payload_size];

    // sizeof(ICMP_ECHO_REPLY) + payload + 8 (ICMP error info) + 256 (headroom)
    let reply_buf_size = mem::size_of::<ICMP_ECHO_REPLY>() + payload_size + 8 + 256;
    let mut reply_buf: Vec<u8> = vec![0u8; reply_buf_size];

    let mut stats = PingStats::new();
    let total_iterations = if infinite { u32::MAX } else { count };
    let session_start = Instant::now();

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    for seq in 0..total_iterations {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        reply_buf.iter_mut().for_each(|b| *b = 0);

        let start = Instant::now();

        let ret = unsafe {
            IcmpSendEcho(
                icmp_handle,
                dest_addr,
                payload.as_ptr() as *mut _,
                payload_size as u16,
                std::ptr::null_mut(),
                reply_buf.as_mut_ptr() as *mut _,
                reply_buf_size as u32,
                timeout_ms,
            )
        };

        let elapsed_ms = start.elapsed().as_millis() as u32;

        if ret == 0 {
            let err = unsafe { GetLastError() };
            if err == 11010 {
                println!("Request timeout for icmp_seq {}", seq);
            } else {
                println!("Request failed for icmp_seq {} (error={})", seq, err);
            }
            stats.record_failure();
        } else {
            let reply = unsafe { &*(reply_buf.as_ptr() as *const ICMP_ECHO_REPLY) };
            let status = reply.Status;

            if status == 0 {
                let ttl = reply.Options.Ttl;
                let rtt = if reply.RoundTripTime == 0 { elapsed_ms } else { reply.RoundTripTime };
                println!(
                    "Reply from {}: bytes={} time={} TTL={}",
                    target_ip, payload_size, format_time(rtt), ttl
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

        if seq + 1 < total_iterations && running.load(std::sync::atomic::Ordering::SeqCst) {
            let elapsed = start.elapsed();
            let target_interval = Duration::from_millis(interval_ms);
            if elapsed < target_interval {
                std::thread::sleep(target_interval - elapsed);
            }
        }
    }

    unsafe {
        IcmpCloseHandle(icmp_handle);
        WSACleanup();
    }

    stats.print_summary();

    if show_session_summary {
        let session_duration = session_start.elapsed();
        let duration_secs = session_duration.as_secs_f64();
        let pps = if duration_secs > 0.0 {
            stats.sent as f64 / duration_secs
        } else {
            0.0
        };

        println!("Session summary:");
        println!("    Duration = {}", format_duration(session_duration));
        println!("    Sent packets in session = {}", stats.sent);
        println!("    Average send rate = {:.2} pkt/s", pps);
    }
}
