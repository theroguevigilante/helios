use helios_parser::Registry;

pub struct FormatDetector<'a> {
    registry: &'a Registry,
}

impl<'a> FormatDetector<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }

    /// Attempts to detect which parser should be used for the given raw log line.
    /// Returns the name of the parser if a match is found.
    pub fn detect(&self, raw_log: &str) -> Option<&'static str> {
        for parser in self.registry.parsers() {
            if parser.detect(raw_log) {
                return Some(parser.name());
            }
        }
        None
    }
}
