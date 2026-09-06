use helios_parser::Registry;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Spawn a background thread that watches `dir` for new/modified `.scm` files
/// and dynamically registers them into the shared registry.
pub fn watch_and_register(
    dir: &Path,
    registry: Arc<RwLock<Registry>>,
) -> anyhow::Result<RecommendedWatcher> {
    let dir_owned = dir.to_path_buf();
    let reg = registry;

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let event = match res {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("File watch error: {}", e);
                return;
            }
        };

        // Only react to Create or Modify events
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {}
            _ => return,
        }

        for path in &event.paths {
            if path.extension().is_some_and(|ext| ext == "scm") {
                tracing::info!("🔄 Detected change: {}", path.display());

                match super::LispParser::from_file(path) {
                    Ok(parser) => {
                        let name = parser.parser_name.clone();
                        match reg.write() {
                            Ok(mut registry) => {
                                registry.register(parser);
                                tracing::info!("🔥 Hot-loaded Lisp parser '{}'", name);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to acquire registry lock: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("❌ Failed to hot-load {}: {}", path.display(), e);
                    }
                }
            }
        }
    })?;

    watcher.watch(&dir_owned, RecursiveMode::NonRecursive)?;
    tracing::info!("👁 Watching {} for new Lisp parsers...", dir_owned.display());
    Ok(watcher)
}
