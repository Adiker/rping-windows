pub struct Config {
    pub count: u32,
    pub timeout_ms: u32,
    pub payload_size: usize,
    pub interval_ms: u64,
    pub show_session_summary: bool,
    pub infinite: bool,
    pub host: String,
}
