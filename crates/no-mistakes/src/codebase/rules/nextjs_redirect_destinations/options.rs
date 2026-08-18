use serde::Deserialize;

pub(super) const DEFAULT_CONFIG_NAMES: &[&str] =
    &["next.config.ts", "next.config.mjs", "next.config.js"];
pub(super) const DEFAULT_APP_ROOT: &str = "app";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) config_path: Option<String>,
    pub(crate) app_root: Option<String>,
    #[serde(default = "default_include_rewrites")]
    pub(crate) include_rewrites: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            config_path: None,
            app_root: None,
            include_rewrites: true,
        }
    }
}

fn default_include_rewrites() -> bool {
    true
}
