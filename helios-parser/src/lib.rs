use helios_core::{Result, UniversalEvent};

pub trait Parser: Send + Sync {
    /// Returns the name of the parser plugin
    fn name(&self) -> &'static str;

    /// Detects if the raw string is parsable by this parser
    fn detect(&self, raw: &str) -> bool;

    /// Parses the raw string into a UniversalEvent
    fn parse(&self, raw: &str) -> Result<UniversalEvent>;

    /// Returns metadata about this parser
    fn metadata(&self) -> ParserMetadata;
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserMetadata {
    pub version: String,
    pub description: String,
    pub author: String,
}

pub struct Registry {
    parsers: Vec<Box<dyn Parser>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register<P: Parser + 'static>(&mut self, parser: P) {
        self.parsers.push(Box::new(parser));
    }

    pub fn parsers(&self) -> &[Box<dyn Parser>] {
        &self.parsers
    }
}
