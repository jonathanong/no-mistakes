fn project_value_import_selectors(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    export_names: &FxHashMap<std::path::PathBuf, ExportNameEdges>,
) -> Vec<Edge> {
    use rayon::prelude::*;
    let indexes = indexes.for_value_imports();
    let mut edges: Vec<_> = inputs
        .graph_files
        .indexable()
        .par_iter()
        .flat_map_iter(|path| value_imports_in_file(inputs, facts, resolver, &indexes, path))
        .collect();
    project_namespace_value_exports(inputs, facts, resolver, &indexes, export_names, &mut edges);
    edges
}

fn value_imports_in_file(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
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
    let projection = ValueImportProjection {
        inputs,
        facts,
        resolver,
        indexes,
    };
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
        if decisions
            .get(import.local.as_str())
            .is_some_and(|found| matches!(found, ImportedSelector::Value) || read.path.is_empty())
        {
            continue;
        }
        let target = targets.entry(import.specifier.as_str()).or_insert_with(|| {
            resolver
                .resolve(&import.specifier, path)
                .and_then(|target| inputs.graph_files.visible_path(&target))
        });
        let Some(target) = *target else { continue };
        let mut target = target.to_path_buf();
        let mut export = if import.kind == ImportedBindingKind::Namespace {
            String::new()
        } else {
            import.imported.clone()
        };
        let mut selector = import.local.clone();
        let mut members = read.path.iter();
        loop {
            let found = decisions.entry(selector.clone()).or_insert_with(|| {
                projection.selector((&target, &export), (path, &selector), &mut edges)
            });
            match found {
                ImportedSelector::Value => break,
                ImportedSelector::Namespace(namespace) => {
                    target = namespace.clone();
                    export.clear();
                }
                ImportedSelector::Missing => break,
            }
            let Some(member) = members
                .next()
                .filter(|member| !member.is_empty() && !member.contains('.'))
            else {
                break;
            };
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

include!("value_import_resolution.rs");
include!("value_import_namespaces.rs");
include!("value_import_queries.rs");
