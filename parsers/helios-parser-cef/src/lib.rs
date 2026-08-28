//! # CEF (Common Event Format) Parser
//!
//! Parses ArcSight Common Event Format logs used by enterprise security vendors
//! including Check Point, Palo Alto, Fortinet, McAfee, and many others.
//!
//! ## Format
//!
//! ```text
//! CEF:Version|Device Vendor|Device Product|Device Version|Signature ID|Name|Severity|Extension
//! ```
//!
//! The extension section contains space-separated `key=value` pairs.
//! Pipes within header fields are escaped as `\|`.

use chrono::{NaiveDateTime, Utc};
use helios_core::{Error, NetworkInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use std::collections::HashMap;

/// The seven fixed header fields in a CEF message.
#[derive(Debug, Clone)]
pub struct CefHeader {
    pub version: String,
    pub device_vendor: String,
    pub device_product: String,
    pub device_version: String,
    pub signature_id: String,
    pub name: String,
    pub severity: String,
}

pub struct CefParser;

impl CefParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CefParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Splits the CEF header respecting `\|` escapes.
/// Returns the 7 header fields and the remaining extension string.
fn split_cef_header(input: &str) -> Option<(CefHeader, &str)> {
    let mut fields: Vec<String> = Vec::with_capacity(7);
    let mut current = String::new();
    let mut chars = input.char_indices().peekable();

    while let Some((_, ch)) = chars.next() {
        if ch == '\\' {
            // Look at next char for escape sequences
            if let Some(&(_, next)) = chars.peek() {
                if next == '|' || next == '\\' {
                    current.push(next);
                    chars.next();
                    continue;
                }
            }
            current.push(ch);
        } else if ch == '|' {
            fields.push(std::mem::take(&mut current));
            // After the 7th field (index 6), the rest is the extension
            if fields.len() == 7 {
                // Collect remaining byte offset
                let remaining_start = chars.peek().map(|(i, _)| *i).unwrap_or(input.len());
                let extension = &input[remaining_start..];
                return Some((
                    CefHeader {
                        version: fields[0].clone(),
                        device_vendor: fields[1].clone(),
                        device_product: fields[2].clone(),
                        device_version: fields[3].clone(),
                        signature_id: fields[4].clone(),
                        name: fields[5].clone(),
                        severity: fields[6].clone(),
                    },
                    extension,
                ));
            }
        } else {
            current.push(ch);
        }
    }

    // If we reach end-of-string with exactly 7 fields already collected
    if fields.len() == 7 {
        return Some((
            CefHeader {
                version: fields[0].clone(),
                device_vendor: fields[1].clone(),
                device_product: fields[2].clone(),
                device_version: fields[3].clone(),
                signature_id: fields[4].clone(),
                name: fields[5].clone(),
                severity: fields[6].clone(),
            },
            "",
        ));
    }

    // If we have 6 fields and remaining text is the 7th (no trailing pipe)
    if fields.len() == 6 {
        fields.push(current);
        return Some((
            CefHeader {
                version: fields[0].clone(),
                device_vendor: fields[1].clone(),
                device_product: fields[2].clone(),
                device_version: fields[3].clone(),
                signature_id: fields[4].clone(),
                name: fields[5].clone(),
                severity: fields[6].clone(),
            },
            "",
        ));
    }

    None
}

/// Parses the CEF extension string into key-value pairs.
///
/// Extension format: `key1=value1 key2=value2 key3=value with spaces`
/// A new key starts when we encounter a token matching `[a-zA-Z0-9]+=`.
fn parse_extension(ext: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if ext.is_empty() {
        return map;
    }

    // Strategy: find all positions where a key= pattern starts.
    // A key is a sequence of alphanumeric chars (and limited special chars)
    // immediately followed by `=`.
    let mut key_positions: Vec<(usize, &str)> = Vec::new();
    let bytes = ext.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Check if this position starts a key (alphanum sequence followed by =)
        if i == 0 || bytes[i - 1] == b' ' {
            let start = i;
            let mut j = i;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j < len && j > start && bytes[j] == b'=' {
                let key = &ext[start..j];
                key_positions.push((j + 1, key)); // value starts after '='
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }

    // Extract values: each value runs from its start to the character before the
    // space preceding the next key
    for (idx, (value_start, key)) in key_positions.iter().enumerate() {
        let value_end = if idx + 1 < key_positions.len() {
            // Find the space before the next key
            let next_value_start = key_positions[idx + 1].0;
            let next_key = key_positions[idx + 1].1;
            // The next key starts at next_value_start - next_key.len() - 1 (the '=')
            // and there should be a space before it
            let next_key_start = next_value_start - next_key.len() - 1;
            if next_key_start > 0 && bytes[next_key_start - 1] == b' ' {
                next_key_start - 1
            } else {
                next_key_start
            }
        } else {
            len
        };

        let value = ext[*value_start..value_end].to_string();
        map.insert(key.to_string(), value);
    }

    map
}

/// Maps CEF severity (0-10 or string) to a normalized severity level.
fn normalize_severity(sev: &str) -> String {
    // CEF severity can be numeric (0-10) or a string
    if let Ok(n) = sev.parse::<u32>() {
        match n {
            0..=3 => "INFO".to_string(),
            4..=6 => "WARN".to_string(),
            7..=8 => "ERROR".to_string(),
            9..=10 => "CRIT".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    } else {
        // Handle string severities
        match sev.to_lowercase().as_str() {
            "low" => "INFO".to_string(),
            "medium" => "WARN".to_string(),
            "high" => "ERROR".to_string(),
            "very-high" | "very high" | "critical" => "CRIT".to_string(),
            _ => sev.to_uppercase(),
        }
    }
}

/// Attempts to parse common CEF timestamp formats.
fn parse_cef_timestamp(ts: &str) -> Option<chrono::DateTime<Utc>> {
    // Epoch millis
    if let Ok(millis) = ts.parse::<i64>() {
        return chrono::DateTime::from_timestamp_millis(millis).map(|dt| dt.with_timezone(&Utc));
    }

    // Common formats used by CEF producers
    let formats = [
        "%b %d %Y %H:%M:%S",     // "Jan 18 2025 11:07:53"
        "%b %d %H:%M:%S",        // "Jan 18 11:07:53"
        "%Y-%m-%dT%H:%M:%S%.fZ", // ISO 8601
        "%Y-%m-%dT%H:%M:%SZ",
    ];

    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(ts, fmt) {
            return Some(dt.and_utc());
        }
    }

    None
}

impl Parser for CefParser {
    fn name(&self) -> &'static str {
        "cef"
    }

    fn detect(&self, raw: &str) -> bool {
        // CEF messages start with "CEF:" optionally preceded by a syslog header.
        // Look for "CEF:" anywhere in the first portion of the line.
        let search_area = if raw.len() > 200 { &raw[..200] } else { raw };
        search_area.contains("CEF:")
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        // Find "CEF:" in the line — there may be a syslog header before it
        let cef_start = raw
            .find("CEF:")
            .ok_or_else(|| Error::ParseError("No CEF: prefix found".into()))?;

        let cef_body = &raw[cef_start + 4..]; // skip "CEF:"

        let (header, ext_str) = split_cef_header(cef_body)
            .ok_or_else(|| Error::ParseError("Failed to parse CEF header fields".into()))?;

        let extensions = parse_extension(ext_str);

        // Build the event
        let timestamp = extensions
            .get("rt")
            .or_else(|| extensions.get("end"))
            .or_else(|| extensions.get("start"))
            .and_then(|ts| parse_cef_timestamp(ts))
            .unwrap_or_else(Utc::now);

        let hostname = extensions
            .get("dhost")
            .or_else(|| extensions.get("shost"))
            .or_else(|| extensions.get("dvchost"))
            .cloned();

        let service = format!("{}|{}", header.device_vendor, header.device_product);

        // Build network info if we have any network fields
        let network = {
            let src_ip = extensions.get("src").cloned();
            let dst_ip = extensions.get("dst").cloned();
            let src_port = extensions.get("spt").and_then(|p| p.parse().ok());
            let dst_port = extensions.get("dpt").and_then(|p| p.parse().ok());
            let proto = extensions.get("proto").cloned();

            if src_ip.is_some() || dst_ip.is_some() {
                Some(NetworkInfo {
                    source_ip: src_ip,
                    destination_ip: dst_ip,
                    source_port: src_port,
                    destination_port: dst_port,
                    protocol: proto,
                })
            } else {
                None
            }
        };

        // Convert remaining extensions to serde_json::Value for attributes
        let attributes: HashMap<String, serde_json::Value> = extensions
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        let message = extensions
            .get("msg")
            .cloned()
            .unwrap_or_else(|| header.name.clone());

        Ok(UniversalEvent {
            timestamp,
            hostname,
            service: Some(service),
            severity: Some(normalize_severity(&header.severity)),
            message,
            raw_event: raw.to_string(),
            network,
            attributes,
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "ArcSight Common Event Format (CEF) Parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cef() {
        let parser = CefParser::new();
        assert!(parser.detect(
            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"
        ));
    }

    #[test]
    fn test_detect_cef_with_syslog_header() {
        let parser = CefParser::new();
        assert!(parser
            .detect("<14>Jan 18 11:07:53 host CEF:0|Vendor|Product|1.0|100|Event|5|src=10.0.0.1"));
    }

    #[test]
    fn test_detect_rejects_non_cef() {
        let parser = CefParser::new();
        assert!(!parser.detect("{\"message\": \"just json\"}"));
        assert!(!parser.detect("<34>Oct 11 22:14:15 mymachine su: something"));
    }

    #[test]
    fn test_parse_basic_cef() {
        let parser = CefParser::new();
        let raw = "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.service.unwrap(), "Security|threatmanager");
        assert_eq!(event.severity.unwrap(), "CRIT"); // severity 10 → CRIT
        assert_eq!(event.message, "worm successfully stopped");

        let net = event.network.unwrap();
        assert_eq!(net.source_ip.unwrap(), "10.0.0.1");
        assert_eq!(net.destination_ip.unwrap(), "2.1.2.2");
        assert_eq!(net.source_port.unwrap(), 1232);
    }

    #[test]
    fn test_parse_cef_with_syslog_prefix() {
        let parser = CefParser::new();
        let raw = "<14>Jan 18 11:07:53 host CEF:0|Check Point|VPN-1|R80|12345|Accept|3|src=192.168.1.5 dst=10.0.0.1 spt=443 dpt=8080 proto=TCP msg=Firewall accepted connection";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.service.unwrap(), "Check Point|VPN-1");
        assert_eq!(event.severity.unwrap(), "INFO"); // severity 3 → INFO
        assert_eq!(event.message, "Firewall accepted connection");

        let net = event.network.unwrap();
        assert_eq!(net.source_ip.unwrap(), "192.168.1.5");
        assert_eq!(net.destination_ip.unwrap(), "10.0.0.1");
        assert_eq!(net.source_port.unwrap(), 443);
        assert_eq!(net.destination_port.unwrap(), 8080);
        assert_eq!(net.protocol.unwrap(), "TCP");
    }

    #[test]
    fn test_parse_cef_medium_severity() {
        let parser = CefParser::new();
        let raw = "CEF:0|Fortinet|FortiGate|6.0|54321|DLP violation|5|src=172.16.0.10 dst=8.8.8.8 msg=Sensitive data detected in outbound traffic";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.severity.unwrap(), "WARN"); // severity 5 → WARN
        assert_eq!(event.service.unwrap(), "Fortinet|FortiGate");
    }

    #[test]
    fn test_parse_cef_with_hostname() {
        let parser = CefParser::new();
        let raw = "CEF:0|PaloAlto|Firewall|10.0|threat|Spyware Detected|8|src=203.0.113.15 dst=10.1.2.50 dhost=workstation-42 dvchost=pa-fw-01";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.hostname.unwrap(), "workstation-42");
        assert_eq!(event.severity.unwrap(), "ERROR"); // severity 8 → ERROR
    }

    #[test]
    fn test_parse_cef_escaped_pipe() {
        let parser = CefParser::new();
        let raw = r"CEF:0|Vendor|Product with \| pipe|1.0|100|Event name|5|src=10.0.0.1";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.service.unwrap(), "Vendor|Product with | pipe");
    }

    #[test]
    fn test_parse_cef_no_extension() {
        let parser = CefParser::new();
        let raw = "CEF:0|Security|Scanner|2.0|200|Port scan detected|7|";
        let event = parser.parse(raw).unwrap();

        assert_eq!(event.message, "Port scan detected");
        assert_eq!(event.severity.unwrap(), "ERROR"); // severity 7 → ERROR
        assert!(event.network.is_none());
    }

    #[test]
    fn test_split_cef_header() {
        let (header, ext) =
            split_cef_header("0|Vendor|Product|1.0|100|Name|5|src=10.0.0.1").unwrap();
        assert_eq!(header.version, "0");
        assert_eq!(header.device_vendor, "Vendor");
        assert_eq!(header.severity, "5");
        assert_eq!(ext, "src=10.0.0.1");
    }

    #[test]
    fn test_parse_extension() {
        let ext = parse_extension("src=10.0.0.1 dst=2.1.2.2 spt=1232 msg=hello world");
        assert_eq!(ext.get("src").unwrap(), "10.0.0.1");
        assert_eq!(ext.get("dst").unwrap(), "2.1.2.2");
        assert_eq!(ext.get("spt").unwrap(), "1232");
        assert_eq!(ext.get("msg").unwrap(), "hello world");
    }

    #[test]
    fn test_normalize_severity() {
        assert_eq!(normalize_severity("0"), "INFO");
        assert_eq!(normalize_severity("3"), "INFO");
        assert_eq!(normalize_severity("5"), "WARN");
        assert_eq!(normalize_severity("8"), "ERROR");
        assert_eq!(normalize_severity("10"), "CRIT");
        assert_eq!(normalize_severity("Low"), "INFO");
        assert_eq!(normalize_severity("High"), "ERROR");
    }
}
