use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct AllowEntry {
    pub(in super::super) path: Option<String>,
    pub(in super::super) project: Option<String>,
    pub(in super::super) reason: String,
}

struct CompiledAllowEntry {
    path: Option<GlobMatcher>,
    project: Option<String>,
}

pub(in super::super) struct CompiledAllowList {
    entries: Vec<CompiledAllowEntry>,
}

impl CompiledAllowList {
    pub(in super::super) fn compile(rule_id: &str, raw: &[AllowEntry]) -> Result<Self> {
        let mut entries = Vec::with_capacity(raw.len());
        for (index, entry) in raw.iter().enumerate() {
            if entry.reason.trim().is_empty() {
                bail!("{rule_id} allow[{index}] requires a non-empty `reason`");
            }
            let path = entry
                .path
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty());
            let project = entry
                .project
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty());
            if path.is_none() && project.is_none() {
                bail!("{rule_id} allow[{index}] must set `path`, `project`, or both");
            }
            let compiled_path = match path {
                Some(pattern) => Some(GlobMatcher::new(
                    &[pattern.to_string()],
                    &format!("{rule_id} allow[{index}].path"),
                )?),
                None => None,
            };
            entries.push(CompiledAllowEntry {
                path: compiled_path,
                project: project.map(str::to_string),
            });
        }
        Ok(Self { entries })
    }

    pub(in super::super) fn is_allowed(&self, rel: &str, project: Option<&str>) -> bool {
        self.entries.iter().any(|entry| {
            entry.path.as_ref().is_none_or(|m| m.is_match(rel))
                && entry.project.as_deref().is_none_or(|p| project == Some(p))
        })
    }
}
