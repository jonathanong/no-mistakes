use super::*;

pub(super) fn defining_modules(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    exported: &str,
    visited: &mut Vec<(PathBuf, String)>,
) -> Vec<UniqueExport> {
    let key = (file.to_path_buf(), exported.to_string());
    if visited.contains(&key) {
        return Vec::new();
    }
    visited.push(key);
    let Some(ts) = facts.get_ts_facts(file) else {
        return Vec::new();
    };
    if ts.parse_error.is_some() || ts.operational_error.is_some() {
        return Vec::new();
    }
    if let Some(local) = local_export(&ts.exported_bindings, exported) {
        return vec![UniqueExport {
            producer: file.to_path_buf(),
            exported: local.to_string(),
        }];
    }
    if let Some(binding) = ts
        .exported_bindings
        .iter()
        .find(|binding| binding.exported == exported && binding.specifier.is_some())
    {
        return unique_import_definitions(graph, facts, file, &binding.local, visited);
    }
    if ts.star_reexport_specifiers.is_empty() {
        return Vec::new();
    }
    unique_import_definitions(graph, facts, file, exported, visited)
}

fn unique_import_definitions(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    exported: &str,
    visited: &mut Vec<(PathBuf, String)>,
) -> Vec<UniqueExport> {
    let mut producers = Vec::new();
    for target in import_files(graph, file) {
        producers.extend(defining_modules(graph, facts, &target, exported, visited));
    }
    producers.sort();
    producers.dedup();
    producers
}
