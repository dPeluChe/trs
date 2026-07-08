use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Compress `ping` output to a single-line summary.
    ///
    /// macOS/BSD and Linux ping share enough structure:
    ///   - first line declares the host/IP
    ///   - N middle lines per packet (reply or timeout)
    ///   - stats block ends with "X packets transmitted, Y received, Z% packet loss"
    ///   - last line carries "min/avg/max/..." timings (variant names per OS)
    ///
    /// We throw away the per-packet noise and keep the summary numbers.
    pub(crate) fn handle_ping(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let mut host = String::new();
        let mut transmitted: Option<u32> = None;
        let mut received: Option<u32> = None;
        let mut loss_pct: Option<f64> = None;
        let mut rtt_min: Option<f64> = None;
        let mut rtt_avg: Option<f64> = None;
        let mut rtt_max: Option<f64> = None;
        let mut error_lines: Vec<String> = Vec::new();

        for raw in input.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }

            // First informational line: "PING <host> (<ip>): ..." (both OSes).
            if host.is_empty() && line.starts_with("PING ") {
                host = line
                    .trim_start_matches("PING ")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                continue;
            }

            // Stats block: "X packets transmitted, Y [packets ]received, Z% packet loss"
            if line.contains("packets transmitted") && line.contains("packet loss") {
                let mut it = line.split(',').map(str::trim);
                if let Some(tx) = it.next() {
                    transmitted = tx.split_whitespace().next().and_then(|n| n.parse().ok());
                }
                if let Some(rx) = it.next() {
                    received = rx.split_whitespace().next().and_then(|n| n.parse().ok());
                }
                if let Some(loss) = it.next() {
                    loss_pct = loss
                        .split_whitespace()
                        .next()
                        .and_then(|n| n.trim_end_matches('%').parse().ok());
                }
                continue;
            }

            // RTT line: "... min/avg/max/... = A/B/C/... ms" (macOS: stddev, Linux: mdev)
            if line.contains("min/avg/max") && line.contains('=') {
                if let Some(nums) = line.split('=').nth(1) {
                    let parts: Vec<&str> = nums
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .split('/')
                        .collect();
                    if parts.len() >= 3 {
                        rtt_min = parts[0].parse().ok();
                        rtt_avg = parts[1].parse().ok();
                        rtt_max = parts[2].parse().ok();
                    }
                }
                continue;
            }

            // Error lines we want to preserve (can't reach host, unknown host, etc.)
            // Not per-packet chatter (those have "bytes from" or "icmp_seq").
            if line.contains("bytes from") || line.contains("icmp_seq") {
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower.contains("unknown host")
                || lower.contains("cannot resolve")
                || lower.contains("network is unreachable")
                || lower.contains("no route to host")
                || lower.contains("operation not permitted")
            {
                error_lines.push(line.to_string());
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "host": host,
                "transmitted": transmitted,
                "received": received,
                "loss_pct": loss_pct,
                "rtt_min_ms": rtt_min,
                "rtt_avg_ms": rtt_avg,
                "rtt_max_ms": rtt_max,
                "errors": error_lines,
            })
            .to_string(),
            _ => format_ping_compact(
                &host,
                transmitted,
                received,
                loss_pct,
                rtt_min,
                rtt_avg,
                rtt_max,
                &error_lines,
                &input,
            ),
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("ping")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn format_ping_compact(
    host: &str,
    transmitted: Option<u32>,
    received: Option<u32>,
    loss_pct: Option<f64>,
    rtt_min: Option<f64>,
    rtt_avg: Option<f64>,
    rtt_max: Option<f64>,
    errors: &[String],
    raw: &str,
) -> String {
    // No stats line — we couldn't classify the output. Pass raw through so
    // the caller still gets SOMETHING (e.g. interrupted ping, unusual OS).
    if transmitted.is_none() {
        return raw.to_string();
    }

    let tx = transmitted.unwrap_or(0);
    let rx = received.unwrap_or(0);
    let loss = loss_pct.unwrap_or_else(|| {
        if tx == 0 {
            0.0
        } else {
            ((tx - rx) as f64 / tx as f64) * 100.0
        }
    });
    let host_label = if host.is_empty() { "?" } else { host };

    let mut line = format!("ping {}  {}/{} ok  {:.0}% loss", host_label, rx, tx, loss);
    if let (Some(a), Some(lo), Some(hi)) = (rtt_avg, rtt_min, rtt_max) {
        line.push_str(&format!("  avg {:.1}ms ({:.1}-{:.1})", a, lo, hi));
    }
    line.push('\n');

    for err in errors {
        line.push_str(&format!("  ! {}\n", err));
    }
    line
}
