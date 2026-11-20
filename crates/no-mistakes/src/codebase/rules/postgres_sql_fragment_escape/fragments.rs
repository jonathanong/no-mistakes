use super::tags::is_fragment_binding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Fragment {
    pub producer: PathBuf,
    pub exported: String,
}

pub(super) struct Catalog<'a> {
    pub(super) fragments: Vec<Fragment>,
    pub(super) sources: Option<&'a crate::codebase::ts_source::SourceStore>,
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
                        });
                    }
                }
            }
        }
        dedup(&mut fragments);
        Self { fragments, sources }
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

    pub(super) fn by_export(&self, name: &str) -> Vec<Fragment> {
        self.fragments
            .iter()
            .filter(|fragment| fragment.exported == name)
            .cloned()
            .collect()
    }
}

pub(super) fn dedup(found: &mut Vec<Fragment>) {
    found.sort();
    found.dedup();
}
