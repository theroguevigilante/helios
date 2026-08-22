use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};

pub struct SyslogParser;

impl SyslogParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for SyslogParser {
    fn name(&self) -> &'static str {
        "syslog"
    }

    fn detect(&self, _raw: &str) -> bool {
        false
    }

    fn parse(&self, _raw: &str) -> Result<UniversalEvent> {
        Err(Error::ParseError("Not implemented".to_string()))
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Syslog log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}
