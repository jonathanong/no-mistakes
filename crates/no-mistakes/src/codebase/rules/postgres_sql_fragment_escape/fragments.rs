use super::tags::is_fragment_binding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::fx::{fx_map, FxHashMap};
use std::path::{Path, PathBuf};

type RelativeIndex = FxHashMap<PathBuf, FxHashMap<String, Vec<PathBuf>>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Fragment {
    pub producer: PathBuf,
    pub exported: String,
    pub open_specifier: Option<String>,
}

pub(super) struct Catalog<'a> {
    pub(super) fragments: Vec<Fragment>,
    pub(super) sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    relative: RelativeIndex,
}

impl<'a> Catalog<'a> {
    pub(super) fn build(
        facts: &CheckFactMap,
        tags: &[TrustedSqlTag],
        sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    ) -> Self {
        let mut fragments = Vec::new();
        for file in facts.graph_file_universe() {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            if ts.operational_error.is_some() || ts.value_flow.incomplete_reason.is_some() {
                continue;
            }
            for binding in &ts.value_flow.bindings {
                if !is_fragment_binding(ts, tags, binding) {
                    continue;
                }
                for export in &ts.exported_bindings {
                    if export.specifier.is_none() && export.local == binding.name {
                        fragments.push(Fragment {
                            producer: file.clone(),
                            exported: export.exported.clone(),
                            open_specifier: None,
                        });
                    }
                }
            }
        }
        dedup(&mut fragments);
        Self {
            fragments,
            sources,
            relative: relative_index(facts),
        }
    }

    pub(super) fn relative_candidates(&self, parent: &Path, stem: &str) -> Vec<PathBuf> {
        self.relative
            .get(parent)
            .and_then(|stems| stems.get(stem))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn local(&self, module: &Path, name: &str) -> Vec<Fragment> {
        self.fragments
            .iter()
            .filter(|fragment| fragment.producer == module && fragment.exported == name)
            .cloned()
            .collect()
    }

    pub(super) fn local_all(&self, module: &Path) -> Vec<Fragment> {
        self.fragments
            .iter()
            .filter(|fragment| fragment.producer == module)
            .cloned()
            .collect()
    }

    #[cfg(test)]
    pub(super) fn by_export(&self, name: &str) -> Vec<Fragment> {
        self.fragments
            .iter()
            .filter(|fragment| fragment.exported == name)
            .cloned()
            .collect()
    }
}

fn relative_index(facts: &CheckFactMap) -> RelativeIndex {
    let mut index = fx_map();
    for file in facts.graph_file_universe() {
        let Some(stem) = file.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Some(parent) = file.parent() else {
            continue;
        };
        push_relative(&mut index, parent, stem, file);
        if stem == "index" {
            let Some(name) = parent.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(grand) = parent.parent() else {
                continue;
            };
            push_relative(&mut index, grand, name, file);
        }
    }
    index
}

fn push_relative(index: &mut RelativeIndex, parent: &Path, stem: &str, file: &Path) {
    index
        .entry(parent.to_path_buf())
        .or_default()
        .entry(stem.to_string())
        .or_default()
        .push(file.to_path_buf());
}

impl Fragment {
    pub(super) fn open(specifier: &str) -> Self {
        Self {
            producer: PathBuf::new(),
            exported: String::new(),
            open_specifier: Some(specifier.to_string()),
        }
    }
}

pub(super) fn dedup(found: &mut Vec<Fragment>) {
    found.sort();
    found.dedup();
}
