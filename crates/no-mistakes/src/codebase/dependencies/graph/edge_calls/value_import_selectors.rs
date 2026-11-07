fn project_value_import_selectors(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    exports: &FxHashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
) -> Vec<Edge> {
    use rayon::prelude::*;
    inputs
        .graph_files
        .indexable()
        .par_iter()
        .flat_map_iter(|path| {
            value_imports_in_file(inputs, facts, resolver, indexes, exports, path)
        })
        .collect()
}

fn value_imports_in_file(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    exports: &FxHashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
    path: &std::path::Path,
) -> Vec<Edge> {
    use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
    let Some(file) = facts
        .get_ts_facts(path)
        .filter(|file| value_import_facts_complete(file))
    else {
        return Vec::new();
    };
    if file.value_flow.catalogs.reads.is_empty() {
        return Vec::new();
    }
    let mut imports: FxHashMap<&str, Option<&ImportedBinding>> = fx_map();
    for import in file
        .imported_bindings
        .iter()
        .filter(|import| !import.is_type_only)
    {
        imports
            .entry(import.local.as_str())
            .and_modify(|binding| *binding = None)
            .or_insert(Some(import));
    }
    if imports.is_empty() {
        return Vec::new();
    }
    let mut targets = fx_map();
    let mut decisions = fx_map();
    let mut edges = Vec::new();
    for read in file
        .value_flow
        .catalogs
        .reads
        .values()
        .filter(|read| read.scope == 0)
    {
        let Some(Some(import)) = imports.get(read.name.as_str()) else {
            continue;
        };
        let namespace = import.kind == ImportedBindingKind::Namespace;
        if namespace && read.path.is_empty() {
            continue;
        }
        if import.imported.contains('.') {
            continue;
        }
        if decisions
            .get(import.local.as_str())
            .is_some_and(|found| *found || read.path.is_empty())
        {
            continue;
        }
        let target = targets.entry(import.specifier.as_str()).or_insert_with(|| {
            resolver
                .resolve(&import.specifier, path)
                .and_then(|target| inputs.graph_files.visible_path(&target))
        });
        let Some(target) = *target else { continue };
        let mut export = if namespace {
            String::new()
        } else {
            import.imported.clone()
        };
        let mut selector = import.local.clone();
        let mut members = read.path.iter();
        loop {
            if !export.is_empty() {
                let found = *decisions.entry(selector.clone()).or_insert_with(|| {
                    let key = (target.to_path_buf(), export.clone());
                    if indexes
                        .export_prerequisites
                        .get(&key)
                        .is_some_and(|failures| !failures.is_empty())
                    {
                        return false;
                    }
                    let Some(ExportedCallableResolution::Value(producer, local)) =
                        exports.get(&key)
                    else {
                        return false;
                    };
                    let Some(source) = indexes.file(facts, producer) else {
                        return false;
                    };
                    if !source.values.immutable_binding(0, local)
                        || !facts
                            .get_ts_facts(producer)
                            .is_some_and(value_import_facts_complete)
                    {
                        return false;
                    }
                    edges.push((
                        NodeId::symbol_in(&inputs.interner, path, &selector),
                        NodeId::symbol_in(&inputs.interner, producer, local),
                        EdgeKind::SymbolImport,
                    ));
                    true
                });
                if found {
                    break;
                }
            }
            let Some(member) = members
                .next()
                .filter(|member| !member.is_empty() && !member.contains('.'))
            else {
                break;
            };
            if !export.is_empty() {
                export.push('.');
            }
            export.push_str(member);
            selector.push('.');
            selector.push_str(member);
        }
    }
    edges
}

fn value_import_facts_complete(file: &crate::codebase::ts_source::facts::TsFileFacts) -> bool {
    file.parse_error.is_none()
        && file.operational_error.is_none()
        && file.value_flow.incomplete_reason.is_none()
}

impl DepGraph {
    /// Canonical live const binding selected by one retained runtime import.
    pub(crate) fn imported_value(
        &self,
        file: &std::path::Path,
        selector: &str,
    ) -> Option<(std::path::PathBuf, String)> {
        let edges = self.dependencies_of_node(&NodeId::symbol(file, selector))?;
        let mut value = None;
        for (target, kind) in edges {
            if *kind == EdgeKind::SymbolImportUnknown {
                return None;
            }
            if *kind != EdgeKind::SymbolImport {
                continue;
            }
            let NodeId::Symbol {
                file,
                symbol,
                callable_id: None,
            } = target
            else {
                return None;
            };
            let candidate = (file.to_path_buf(), symbol.to_string());
            if value.as_ref().is_some_and(|value| value != &candidate) {
                return None;
            }
            value = Some(candidate);
        }
        value
    }
}
