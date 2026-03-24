pub fn icmp_status_to_str(status: u32) -> &'static str {
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
