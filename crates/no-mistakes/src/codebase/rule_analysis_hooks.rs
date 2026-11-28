use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

type PrepareRequestedRuleConfig = fn(
    &Path,
    &str,
    &NoMistakesConfig,
    &[PathBuf],
    &SourceStore,
) -> Result<Option<NoMistakesConfig>>;

pub(crate) struct Hooks {
    pub(crate) prepare_requested_rule_config: PrepareRequestedRuleConfig,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn prepare_requested_rule_config(
    root: &Path,
    rule_id: &str,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Option<NoMistakesConfig>> {
    let hooks = HOOKS
        .get()
        .expect("rule analysis hooks registered by the facade");
    (hooks.prepare_requested_rule_config)(root, rule_id, config, files, sources)
}
