#![allow(clippy::arc_with_non_send_sync)]
mod builtins;
mod engine;
pub mod validate;
pub mod watcher;

use helios_core::{Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use validate::{validate_script, ValidationResult};
pub use watcher::watch_and_register;

/// A parser backed by a Steel Scheme script.
pub struct LispParser {
    pub(crate) parser_name: String,
    description: String,
    script_path: PathBuf,
    engine: Arc<Mutex<steel::steel_vm::engine::Engine>>,
}

impl LispParser {
    /// Load a `.scm` file. Validates before constructing.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        use steel::rvals::SteelVal;

        // Step 1: Validate the script first
        let result = validate_script(path)?;
        if !result.is_valid {
            anyhow::bail!("Validation failed: {}", result.errors.join(", "));
        }

        // Step 2: Create engine and load
        let mut eng = engine::create_engine();
        let source = std::fs::read_to_string(path)?;
        // Steel 0.8 requires Into<Cow<'static, str>>. We leak the string since
        // parser scripts live for the entire program lifetime.
        let source_static: &'static str = Box::leak(source.into_boxed_str());
        eng.compile_and_run_raw_program(source_static)?;

        let name_val = eng.call_function_by_name_with_args("parser-name", vec![])?;
        let name = match name_val {
            SteelVal::StringV(s) => s.to_string(),
            _ => anyhow::bail!("(parser-name) must return a string"),
        };

        let description = eng
            .call_function_by_name_with_args("parser-description", vec![])
            .ok()
            .and_then(|v| match v {
                SteelVal::StringV(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| {
                format!(
                    "Lisp parser: {}",
                    path.file_name().unwrap().to_string_lossy()
                )
            });

        tracing::info!("✅ Loaded Lisp parser '{}' from {}", name, path.display());

        Ok(Self {
            parser_name: name,
            description,
            script_path: path.to_path_buf(),
            engine: Arc::new(Mutex::new(eng)),
        })
    }

    /// Returns the script path (for display/debugging).
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }
}

// SAFETY: Steel engine access is serialized via Mutex.
unsafe impl Send for LispParser {}
unsafe impl Sync for LispParser {}

impl Parser for LispParser {
    fn name(&self) -> &'static str {
        // Leak the string to get a 'static lifetime.
        // This is acceptable — parsers live for the entire program lifetime.
        Box::leak(self.parser_name.clone().into_boxed_str())
    }

    fn detect(&self, raw: &str) -> bool {
        use steel::rvals::SteelVal;
        let mut eng = self.engine.lock().unwrap();
        eng.call_function_by_name_with_args("detect", vec![SteelVal::StringV(raw.into())])
            .map(|v| v.is_truthy())
            .unwrap_or(false)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        use steel::rvals::SteelVal;
        let mut eng = self.engine.lock().unwrap();
        let result = eng
            .call_function_by_name_with_args("parse", vec![SteelVal::StringV(raw.into())])
            .map_err(|e| helios_core::Error::ParseError(e.to_string()))?;
        engine::steel_value_to_event(result, raw)
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: "1.0.0".to_string(),
            description: self.description.clone(),
            author: "Lisp Extension".to_string(),
        }
    }
}

/// Scan a directory for `.scm` files, validate each, and return valid parsers.
pub fn load_lisp_parsers(dir: &Path) -> Vec<LispParser> {
    let mut parsers = Vec::new();
    if !dir.exists() {
        tracing::debug!("Lisp parser directory not found: {}", dir.display());
        return parsers;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to read Lisp parser dir: {}", e);
            return parsers;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "scm") {
            match LispParser::from_file(&path) {
                Ok(p) => parsers.push(p),
                Err(e) => tracing::warn!("❌ Skipping {}: {}", path.display(), e),
            }
        }
    }

    tracing::info!(
        "Loaded {} Lisp parser(s) from {}",
        parsers.len(),
        dir.display()
    );
    parsers
}

/// Resolve the default Lisp parsers directory.
pub fn default_lisp_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".helios/parsers");
        if path.exists() {
            return path;
        }
    }
    PathBuf::from("parsers/lisp")
}
