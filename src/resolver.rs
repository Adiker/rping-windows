use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

pub fn resolve_host(host: &str) -> Option<Ipv4Addr> {
    let addr_str = format!("{}:0", host);
    addr_str.to_socket_addrs().ok()?.find_map(|addr| {
        if let IpAddr::V4(ip) = addr.ip() {
            Some(ip)
        } else {
            None
        }
    })
}
