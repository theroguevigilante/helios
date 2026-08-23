use chrono::Utc;
use helios_core::{ProcessInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use syslog_loose::parse_message;

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
        // A simple heuristic: try to parse it with syslog_loose.
        // If it successfully extracts a severity or facility from a PRI header, it's syslog.
        let msg = parse_message(raw, syslog_loose::Variant::Either);
        msg.severity.is_some() || msg.facility.is_some()
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
    fn test_detect_syslog() {
        let parser = SyslogParser::new();
        let raw = "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        assert!(parser.detect(raw));

        let non_syslog = "{\"message\": \"just some json\"}";
        assert!(!parser.detect(non_syslog));
    }

    #[test]
    fn test_parse_syslog() {
        let parser = SyslogParser::new();
        let raw = "<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] BOMAn application event log entry...";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.unwrap(), "mymachine.example.com");
        assert_eq!(event.service.unwrap(), "evntslog");
        assert_eq!(event.message, "BOMAn application event log entry...");
        assert_eq!(event.severity.unwrap(), "NOTICE");
    }
}
