use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};

pub struct ApacheParser;

impl Default for ApacheParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ApacheParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for ApacheParser {
    fn name(&self) -> &'static str {
        "apache"
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
            description: "Apache log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}
