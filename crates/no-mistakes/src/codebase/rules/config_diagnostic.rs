//! Authored file and line for a config diagnostic.
//!
//! A config with no loaded path stays `.no-mistakes.yml:1`. Programmatic
//! values must not borrow a neighboring file on disk.

use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

const FALLBACK_FILE: &str = ".no-mistakes.yml";

pub(crate) struct ConfigAnchor {
    file: String,
    locations: Option<Arc<ConfigLocations>>,
    index: Option<usize>,
}

impl ConfigAnchor {
    pub(crate) fn resolve(
        root: &Path,
        config: &NoMistakesConfig,
        sources: Option<&SourceStore>,
        rule: &RuleDef,
    ) -> Self {
        let Some(path) = config.loaded_config_path.0.as_deref() else {
            return Self::fallback();
        };
        let file = config
            .loaded_config_path
            .relative_to(root)
            .unwrap_or_else(|| FALLBACK_FILE.to_string());
        // SourceStore owns the read. Without the request store the path is
        // still reported and the line stays 1.
        let locations = sources.and_then(|sources| sources.config_locations(path));
        // `rule_applications` borrows `config.rules`, so identity is the
        // authored sequence index. A clone is not an authored entry.
        let index = config
            .rules
            .iter()
            .position(|candidate| std::ptr::eq(candidate, rule));
        Self {
            file,
            locations,
            index,
        }
    }

    pub(crate) fn fallback() -> Self {
        Self {
            file: FALLBACK_FILE.to_string(),
            locations: None,
            index: None,
        }
    }

    pub(crate) fn option_line(&self, key: &str) -> usize {
        self.located(key, None)
    }

    pub(crate) fn option_item_line(&self, key: &str, item: usize) -> usize {
        self.located(key, Some(item))
    }

    pub(crate) fn nested_option_item_line(
        &self,
        key: &str,
        index: usize,
        nested_key: &str,
        item: usize,
    ) -> usize {
        let Some(rule) = self.index else { return 1 };
        let path = ConfigPath::default()
            .key("rules")
            .index(rule)
            .key("options")
            .key(key)
            .index(index)
            .key(nested_key)
            .index(item);
        self.locations
            .as_deref()
            .and_then(|locations| locations.line(&path))
            .unwrap_or_else(|| self.option_item_line(key, index))
    }

    pub(crate) fn listed_line(&self, key: &str, values: &[String], value: &str) -> usize {
        match values.iter().position(|item| item == value) {
            Some(index) => self.option_item_line(key, index),
            None => self.option_line(key),
        }
    }

    pub(crate) fn invalid_glob_line(&self, key: &str, patterns: &[String]) -> usize {
        match patterns
            .iter()
            .position(|pattern| globset::Glob::new(pattern).is_err())
        {
            Some(index) => self.option_item_line(key, index),
            None => self.option_line(key),
        }
    }

    pub(crate) fn emit(
        &self,
        rule_id: &str,
        line: usize,
        message: String,
        import: Option<String>,
        target: Option<String>,
    ) -> RuleFinding {
        RuleFinding {
            rule: rule_id.to_string(),
            file: self.file.clone(),
            line,
            message,
            import,
            target,
        }
    }

    fn located(&self, key: &str, item: Option<usize>) -> usize {
        let Some(locations) = self.locations.as_deref() else {
            return 1;
        };
        let Some(index) = self.index else {
            return 1;
        };
        let mut path = ConfigPath::default()
            .key("rules")
            .index(index)
            .key("options")
            .key(key);
        if let Some(item) = item {
            path = path.index(item);
        }
        if let Some(line) = locations.line(&path) {
            return line;
        }
        // A missing key is reported on the rule entry, not on line 1.
        let rule = ConfigPath::default().key("rules").index(index);
        locations
            .line(&rule.clone().key("rule"))
            .or_else(|| locations.line(&rule))
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests;
