use super::import_resolution::FragmentResolver;
use super::tags::is_fragment_binding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::fx::{fx_map, FxHashMap};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::path::{Path, PathBuf};

type RelativeIndex = FxHashMap<PathBuf, FxHashMap<String, Vec<PathBuf>>>;
type FragmentIndex = FxHashMap<PathBuf, FxHashMap<String, Vec<Fragment>>>;
type ProducerIndex = FxHashMap<PathBuf, Vec<Fragment>>;
type AmbientIndex = FxHashMap<String, Vec<PathBuf>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Fragment {
    pub producer: PathBuf,
    pub exported: String,
}

pub(super) struct Catalog<'a> {
    pub(super) sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    relative: RelativeIndex,
    pub(super) by_export: FragmentIndex,
    by_producer: ProducerIndex,
    ambient: AmbientIndex,
    pub(super) resolution: Option<&'a dyn FragmentResolver>,
    pub(super) tags: &'a [TrustedSqlTag],
    unresolved: DashMap<PathBuf, String>,
}

impl<'a> Catalog<'a> {
    pub(super) fn build(
        facts: &CheckFactMap,
        tags: &'a [TrustedSqlTag],
        sources: Option<&'a crate::codebase::ts_source::SourceStore>,
        resolution: Option<&'a dyn FragmentResolver>,
    ) -> Self {
        let mut fragments = Vec::new();
        for file in facts.graph_file_universe() {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
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
        let (by_export, by_producer) = index_fragments(&fragments);
        Self {
            sources,
            relative: if resolution.is_none() {
                relative_index(facts)
            } else {
                fx_map()
            },
            by_export,
            by_producer,
            ambient: ambient_index(facts, sources),
            resolution,
            tags,
            unresolved: DashMap::new(),
        }
    }

    pub(super) fn record_unresolved(&self, importer: &Path, specifier: &str) {
        match self.unresolved.entry(importer.to_path_buf()) {
            Entry::Occupied(mut entry) if specifier < entry.get().as_str() => {
                entry.insert(specifier.to_owned());
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(specifier.to_owned());
            }
        }
    }

    pub(super) fn take_unresolved(&mut self) -> DashMap<PathBuf, String> {
        std::mem::take(&mut self.unresolved)
    }

    pub(super) fn ambient_files(&self, specifier: &str) -> Vec<PathBuf> {
        self.ambient.get(specifier).cloned().unwrap_or_default()
    }

    pub(super) fn relative_candidates(&self, parent: &Path, stem: &str) -> Vec<PathBuf> {
        self.relative
            .get(parent)
            .and_then(|stems| stems.get(stem))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn local(&self, module: &Path, name: &str) -> Vec<Fragment> {
        self.by_export
            .get(module)
            .and_then(|names| names.get(name))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn local_all(&self, module: &Path) -> Vec<Fragment> {
        self.by_producer.get(module).cloned().unwrap_or_default()
    }
}

fn index_fragments(fragments: &[Fragment]) -> (FragmentIndex, ProducerIndex) {
    let mut by_export: FragmentIndex = fx_map();
    let mut by_producer: ProducerIndex = fx_map();
    for fragment in fragments {
        by_export
            .entry(fragment.producer.clone())
            .or_default()
            .entry(fragment.exported.clone())
            .or_default()
            .push(fragment.clone());
        by_producer
            .entry(fragment.producer.clone())
            .or_default()
            .push(fragment.clone());
    }
    (by_export, by_producer)
}

fn ambient_index(
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> AmbientIndex {
    let mut index: AmbientIndex = fx_map();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        let Some(source) = super::source_text::file_text(ts, sources, file) else {
            continue;
        };
        for specifier in super::modules::declare_module_specifiers(source.as_ref()) {
            index.entry(specifier).or_default().push(file.clone());
        }
    }
    for files in index.values_mut() {
        files.sort();
        files.dedup();
    }
    index
}

fn relative_index(facts: &CheckFactMap) -> RelativeIndex {
    let mut index: RelativeIndex = fx_map();
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

pub(super) fn dedup(found: &mut Vec<Fragment>) {
    found.sort();
    found.dedup();
}
