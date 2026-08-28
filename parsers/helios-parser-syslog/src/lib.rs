use chrono::Utc;
use helios_core::{ProcessInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use syslog_loose::parse_message;

const BSD_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Checks whether the line looks like a BSD-style syslog message without a PRI
/// header, e.g. `Oct 11 22:14:15 mymachine su: ...`.
///
/// The heuristic matches: `<Month> <Day> <HH:MM:SS> <hostname>`.
fn looks_like_bsd_syslog(raw: &str) -> bool {
    let bytes = raw.as_bytes();

    // Must start with a three-letter month abbreviation
    if bytes.len() < 16 {
        return false;
    }

    let month = &raw[..3];
    if !BSD_MONTHS.contains(&month) {
        return false;
    }

    // Followed by a space, one-or-two digit day, space, HH:MM:SS pattern
    // Example: "Oct 11 22:14:15" or "Oct  1 22:14:15"
    if bytes[3] != b' ' {
        return false;
    }

    // Find the timestamp end (after HH:MM:SS)
    // Skip the space(s) after the month and collect remaining parts.
    // BSD syslog uses "Oct 11 22:14:15" or "Oct  5 03:02:01" (double space for single-digit day).
    let rest = raw[4..].trim_start();
    let parts: Vec<&str> = rest.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return false;
    }

    // parts[0] = day, parts[1] = HH:MM:SS, parts[2] = rest of message
    let time = parts[1];
    if time.len() != 8 {
        return false;
    }

    let time_bytes = time.as_bytes();
    time_bytes[2] == b':' && time_bytes[5] == b':'
}

pub struct SyslogParser;

impl SyslogParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SyslogParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for SyslogParser {
    fn name(&self) -> &'static str {
        "syslog"
    }

    fn detect(&self, raw: &str) -> bool {
        let msg = parse_message(raw, syslog_loose::Variant::Either);

        // Case 1: PRI header is present — syslog_loose extracts severity/facility
        if msg.severity.is_some() || msg.facility.is_some() {
            return true;
        }

        // Case 2: No PRI header, but the line matches BSD syslog timestamp pattern
        looks_like_bsd_syslog(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let parsed = parse_message(raw, syslog_loose::Variant::Either);

        let timestamp = parsed
            .timestamp
            .map(|ts| ts.into())
            .unwrap_or_else(Utc::now);

        let mut event = UniversalEvent {
            timestamp,
            hostname: parsed.hostname.map(|s| s.to_string()),
            service: parsed.appname.map(|s| s.to_string()),
            severity: parsed
                .severity
                .map(|s| s.as_str().to_string().to_uppercase()),
            message: parsed.msg.to_string(),
            raw_event: raw.to_string(),
            ..Default::default()
        };

        if let Some(procid) = parsed.procid {
            let pid = match procid {
                syslog_loose::ProcId::PID(p) => Some(p as u32),
                syslog_loose::ProcId::Name(_) => None,
            };

            event.process = Some(ProcessInfo {
                pid,
                name: parsed.appname.map(|s| s.to_string()),
                thread_id: None,
                thread_name: None,
            });
        }

        Ok(event)
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "RFC 3164 and RFC 5424 Syslog Parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_with_pri() {
        let parser = SyslogParser::new();
        let raw = "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        assert!(parser.detect(raw));
    }

    #[test]
    fn test_detect_without_pri() {
        let parser = SyslogParser::new();
        let raw = "Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        assert!(parser.detect(raw));
    }

    #[test]
    fn test_detect_without_pri_single_digit_day() {
        let parser = SyslogParser::new();
        let raw = "Jan  5 03:02:01 myhost sshd[1234]: Accepted publickey for user";
        assert!(parser.detect(raw));
    }

    #[test]
    fn test_detect_rejects_json() {
        let parser = SyslogParser::new();
        let raw = "{\"message\": \"just some json\"}";
        assert!(!parser.detect(raw));
    }

    #[test]
    fn test_detect_rejects_plain_text() {
        let parser = SyslogParser::new();
        let raw = "This is just a random line of text";
        assert!(!parser.detect(raw));
    }

    #[test]
    fn test_parse_rfc5424() {
        let parser = SyslogParser::new();
        let raw = "<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] BOMAn application event log entry...";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.unwrap(), "mymachine.example.com");
        assert_eq!(event.service.unwrap(), "evntslog");
        assert_eq!(event.message, "BOMAn application event log entry...");
        assert_eq!(event.severity.unwrap(), "NOTICE");
    }

    #[test]
    fn test_parse_without_pri() {
        let parser = SyslogParser::new();
        let raw = "Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.unwrap(), "mymachine");
        assert_eq!(event.service.unwrap(), "su");
        assert!(event.message.contains("su root"));
        assert_eq!(event.raw_event, raw);
    }

    #[test]
    fn test_parse_cisco_asa_without_pri() {
        let parser = SyslogParser::new();
        let raw =
            "Jan 24 12:34:56 firewall1 %ASA-4-106023: Deny tcp src outside:192.168.1.1/1234 dst inside:10.0.0.1/80";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.unwrap(), "firewall1");
        assert_eq!(event.service.unwrap(), "%ASA-4-106023");
    }

    #[test]
    fn test_parse_mac_syslog() {
        let parser = SyslogParser::new();
        let raw = "Aug 28 10:23:45 macbook kernel[0]: message text";
        assert!(parser.detect(raw));
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.as_deref(), Some("macbook"));
        assert_eq!(event.service.as_deref(), Some("kernel"));
        assert_eq!(event.message, "message text");
        assert_eq!(event.process.as_ref().and_then(|p| p.pid), Some(0));
    }
}

