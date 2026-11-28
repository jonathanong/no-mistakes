use std::sync::Arc;

#[doc(hidden)]
pub use no_mistakes_ts_analysis::source_access::{JsonLoadError, JsonParseOutcome};

impl super::SourceStore {
    #[doc(hidden)]
    pub fn parse_json_path(&self, path: &std::path::Path) -> JsonParseOutcome {
        let path = crate::codebase::ts_source::normalize_discovery_path(path);
        let cell = super::once_lock_slot(&self.json_parses, path.clone());
        self.increment("manifest.requests", 1);
        let parsed = std::cell::Cell::new(false);
        let result = cell
            .get_or_init(|| {
                parsed.set(true);
                self.json_parse_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.increment("manifest.parses", 1);
                match self.read_path(&path) {
                    // Package managers accept one BOM; retain the exact cached source.
                    Ok(source) => {
                        serde_json::from_str(source.strip_prefix('\u{FEFF}').unwrap_or(&source))
                            .map(Arc::new)
                            .map_err(|error| {
                                self.increment("manifest.errors", 1);
                                JsonLoadError::Syntax(Arc::new(error))
                            })
                    }
                    Err(error) => {
                        self.increment("manifest.errors", 1);
                        Err(JsonLoadError::Io(error))
                    }
                }
            })
            .clone();
        if !parsed.get() {
            self.increment("manifest.cache_hits", 1);
        }
        result
    }

    #[doc(hidden)]
    pub fn json_parse_count(&self) -> usize {
        self.json_parse_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
