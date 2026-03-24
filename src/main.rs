use std::env;

mod cli;
mod config;
mod format;
mod icmp_status;
mod ping;
mod resolver;
mod stats;
mod win_icmp;

use cli::{parse_args, print_usage};
use format::format_duration;
use ping::run_ping;
use resolver::resolve_host;
use win_icmp::WsaSession;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    if args.len() < 2 {
        print_usage(program);
        std::process::exit(1);
    }

    let config = match parse_args(&args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            print_usage(program);
            std::process::exit(1);
        }
    };

    let _wsa_session = match WsaSession::startup() {
        Ok(session) => session,
        Err(err) => {
            eprintln!("WSAStartup failed: {}", err);
            std::process::exit(1);
        }
    };

    let target_ip = match resolve_host(&config.host) {
        Some(ip) => ip,
        None => {
            eprintln!(
                "Ping: cannot resolve '{}': Name or service not known.",
                config.host
            );
            std::process::exit(1);
        }
    };

    let display_host = if config.host == target_ip.to_string() {
        config.host.clone()
    } else {
        format!("{} [{}]", config.host, target_ip)
    };

    println!(
        "\nPinging {} with {} bytes of data:",
        display_host, config.payload_size
    );

    let session = match run_ping(target_ip, &config) {
        Ok(session) => session,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    session.stats.print_summary();

    if config.show_session_summary {
        let duration_secs = session.duration.as_secs_f64();
        let pps = if duration_secs > 0.0 {
            session.stats.sent as f64 / duration_secs
        } else {
            0.0
        };

        println!("Session summary:");
        println!("    Duration = {}", format_duration(session.duration));
        println!("    Sent packets in session = {}", session.stats.sent);
        println!("    Average send rate = {:.2} pkt/s", pps);
    }
}
