use crate::codebase::structured_value::expand_yaml_mapping_if_merged;
use serde_yaml::Value;
use std::borrow::Cow;

/// The workflow root plus the `on` and `concurrency` sections, each with its
/// merge keys expanded.
pub(super) struct MergedSections<'a> {
    pub(super) root: Cow<'a, Value>,
    pub(super) on: Option<Value>,
    pub(super) concurrency: Option<Value>,
}

/// Expand merge keys once per section. A merge that cannot be applied is
/// returned as a message for a malformed-workflow diagnostic.
pub(super) fn merged_sections(value: &Value) -> Result<MergedSections<'_>, String> {
    let root = expand_yaml_mapping_if_merged(value)?;
    let on = section(&root, "on")?;
    let concurrency = section(&root, "concurrency")?;
    Ok(MergedSections {
        root,
        on,
        concurrency,
    })
}

fn section(root: &Value, key: &str) -> Result<Option<Value>, String> {
    root.get(key)
        .map(|value| expand_yaml_mapping_if_merged(value).map(Cow::into_owned))
        .transpose()
}
