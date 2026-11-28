use anyhow::Result;
use serde_json::{value::RawValue, Value};
use std::path::{Path, PathBuf};

pub(super) use crate::options::parse_options;

pub(super) fn report_value(raw: &RawValue) -> Value {
    serde_json::from_str(raw.get()).expect("report JSON is valid")
}

pub(super) fn resolve_tsconfig(
    root: &Path,
    explicit: Option<&str>,
    visible_paths: &[PathBuf],
) -> Result<no_mistakes::codebase::dependencies::TsConfig> {
    let explicit = explicit.map(Path::new);
    no_mistakes::codebase::ts_resolver::resolve_tsconfig_from_visible(explicit, root, visible_paths)
}
