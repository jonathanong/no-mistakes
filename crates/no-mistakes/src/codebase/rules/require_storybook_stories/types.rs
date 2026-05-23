use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct Options {
    pub(super) stories: Vec<String>,
    pub(super) include: Vec<String>,
    pub(super) exclude: Vec<String>,
    #[serde(alias = "include_all_react_named_exports")]
    pub(super) include_all_react_named_exports: bool,
    #[serde(alias = "include_all_react_default_exports")]
    pub(super) include_all_react_default_exports: bool,
    #[serde(alias = "required_props")]
    pub(super) required_props: Vec<String>,
    #[serde(alias = "allow_components")]
    pub(super) allow_components: BTreeMap<String, String>,
    #[serde(alias = "allow_files")]
    pub(super) allow_files: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub(super) struct Component {
    pub(super) key: String,
    pub(super) file: PathBuf,
    pub(super) repo_file: String,
    pub(super) project_file: String,
    pub(super) export_name: String,
    pub(super) line: usize,
    pub(super) explicit: bool,
}

pub(super) fn component_key(file: &str, export_name: &str) -> String {
    format!("{file}#{export_name}")
}

pub(super) fn is_react_source_file(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("tsx" | "jsx")
    )
}

pub(super) fn source_has_required_prop(source: &str, opts: &Options) -> bool {
    opts.required_props.iter().any(|prop| {
        source.contains(&format!("'{prop}'"))
            || source.contains(&format!("\"{prop}\""))
            || source.contains(&format!("{prop}="))
    })
}

pub(super) struct GlobMatcher {
    globset: Option<GlobSet>,
}

impl GlobMatcher {
    pub(super) fn new<'a>(patterns: impl IntoIterator<Item = &'a String>) -> Self {
        let mut builder = GlobSetBuilder::new();
        let mut added = 0usize;
        for pattern in patterns {
            if let Ok(glob) = GlobBuilder::new(pattern.trim_start_matches("./"))
                .literal_separator(false)
                .build()
            {
                builder.add(glob);
                added += 1;
            }
        }
        Self {
            globset: (added > 0).then(|| builder.build().ok()).flatten(),
        }
    }

    pub(super) fn is_match(&self, path: &str) -> bool {
        self.globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(path))
    }
}
