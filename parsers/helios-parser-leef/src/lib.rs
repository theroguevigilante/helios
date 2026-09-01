use chrono::Utc;
use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};

pub struct LeefParser;

impl LeefParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LeefParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for LeefParser {
    fn name(&self) -> &'static str {
        "leef"
    }

    fn detect(&self, raw: &str) -> bool {
        let search_area = if raw.len() > 200 { &raw[..200] } else { raw };
        search_area.contains("LEEF:1.0|") || search_area.contains("LEEF:2.0|")
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let leef_start = raw
            .find("LEEF:")
            .ok_or_else(|| Error::ParseError("No LEEF: prefix found".into()))?;

        let leef_body = &raw[leef_start..];
        let mut parts = leef_body.splitn(6, '|');

        let version_str = parts.next().unwrap_or(""); // LEEF:1.0 or LEEF:2.0
        let vendor = parts.next().unwrap_or("");
        let product = parts.next().unwrap_or("");
        let _version = parts.next().unwrap_or("");
        let event_id = parts.next().unwrap_or("");

        let remainder = parts.next().unwrap_or("");

        // LEEF 2.0 has a delimiter field, LEEF 1.0 does not
        let (delimiter, _extensions) = if version_str == "LEEF:2.0" {
            let mut rem_parts = remainder.splitn(2, '|');
            let delim = rem_parts.next().unwrap_or("\t");
            let ext = rem_parts.next().unwrap_or("");
            (delim, ext)
        } else {
            ("\t", remainder) // Default delimiter for LEEF 1.0 is tab
        };

        // Standardize delimiter (e.g. x09 or ^ or \t)
        let actual_delim = match delimiter {
            "x09" | "\\t" | "0x09" => "\t",
            "x5e" => "^",
            other => other,
        };

        let service = if !vendor.is_empty() && !product.is_empty() {
            format!("{} {}", vendor, product)
        } else {
            "unknown".to_string()
        };

        // For this simple implementation, we won't fully parse all k=v extensions,
        // but we assume standard severity and message defaults.
        // A full parser would split by `actual_delim` and map keys (like 'sev', 'devTime').

        let severity = if remainder.contains("sev=10") || remainder.contains("sev=9") {
            "CRIT".to_string()
        } else if remainder.contains("sev=8") || remainder.contains("sev=7") {
            "ERROR".to_string()
        } else if remainder.contains("sev=6") || remainder.contains("sev=5") {
            "WARN".to_string()
        } else if remainder.contains("sev=4") || remainder.contains("sev=3") {
            "INFO".to_string()
        } else {
            "INFO".to_string()
        };

        Ok(UniversalEvent {
            timestamp: Utc::now(), // In a full implementation, parse 'devTime' extension
            hostname: None,
            service: Some(service),
            severity: Some(severity),
            message: event_id.to_string(),
            raw_event: raw.to_string(),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Native IBM QRadar LEEF format parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}
