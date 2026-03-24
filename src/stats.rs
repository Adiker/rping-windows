use crate::format::format_time;

pub struct PingStats {
    pub sent: u32,
    pub received: u32,
    pub times: Vec<u32>,
}

impl PingStats {
    pub fn new() -> Self {
        PingStats {
            sent: 0,
            received: 0,
            times: Vec::new(),
        }
    }

    pub fn record_success(&mut self, ms: u32) {
        self.sent += 1;
        self.received += 1;
        self.times.push(ms);
    }

    pub fn record_failure(&mut self) {
        self.sent += 1;
    }

    pub fn print_summary(&self) {
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
