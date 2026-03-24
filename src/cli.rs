use crate::config::Config;

pub fn print_usage(program: &str) {
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

pub fn parse_args(args: &[String]) -> Result<Config, String> {
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
        return Err("No host specified.".to_string());
    }

    Ok(Config {
        count,
        timeout_ms,
        payload_size,
        interval_ms,
        show_session_summary,
        infinite,
        host,
    })
}
