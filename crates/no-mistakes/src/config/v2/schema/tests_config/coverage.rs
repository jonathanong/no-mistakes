use serde::{Deserialize, Serialize};

/// Which Playwright coverage findings to emit. Defaults keep both gates on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaywrightCoverageConfig {
    #[serde(default = "default_true")]
    pub routes: bool,
    #[serde(default = "default_true")]
    pub selectors: bool,
}

impl Default for PlaywrightCoverageConfig {
    fn default() -> Self {
        Self {
            routes: true,
            selectors: true,
        }
    }
}

fn default_true() -> bool {
    true
}
