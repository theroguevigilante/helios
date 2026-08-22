use helios_core::{Result, UniversalEvent};

/// Trait for enriching parsed events with additional context
pub trait Enricher: Send + Sync {
    /// Name of the enricher
    fn name(&self) -> &'static str;

    /// Mutates the event in place to add metadata, process info, etc.
    fn enrich(&self, event: &mut UniversalEvent) -> Result<()>;
}

pub struct EnrichmentPipeline {
    enrichers: Vec<Box<dyn Enricher>>,
}

impl EnrichmentPipeline {
    pub fn new() -> Self {
        Self {
            enrichers: Vec::new(),
        }
    }

    pub fn add_enricher<E: Enricher + 'static>(&mut self, enricher: E) {
        self.enrichers.push(Box::new(enricher));
    }

    pub fn run(&self, event: &mut UniversalEvent) -> Result<()> {
        for enricher in &self.enrichers {
            enricher.enrich(event)?;
        }
        Ok(())
    }
}
