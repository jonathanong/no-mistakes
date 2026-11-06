use super::*;

pub(super) fn resolve_component_key(
    project_root: &Path,
    shared: &CheckFactMap,
    file: &Path,
    export_name: &str,
    component_keys: &HashSet<String>,
    resolver: &dyn ImportResolution,
    state: (
        &mut HashSet<(PathBuf, String)>,
        Option<&super::super::source_failures::SourceFailures<'_>>,
    ),
) -> Option<String> {
    let (visiting, demand) = state;
    if !file.starts_with(project_root) {
        return None;
    }
    if let Some(demand) = demand {
        demand.require(file);
    }
    let project_file = relative_slash_path(project_root, file);
    let direct = component_key(&project_file, export_name);
    if component_keys.contains(&direct) {
        return Some(direct);
    }
    if !visiting.insert((file.to_path_buf(), export_name.to_string())) {
        return None;
    }
    if let Some(demand) = demand {
        demand.require_symbols(file);
    }
    let facts = shared.ts.get(file)?;
    let symbols = facts.symbols.as_ref()?;
    for export in &symbols.exports {
        let ExportKind::ReExport { source, imported } = &export.kind else {
            continue;
        };
        if export.is_type_only {
            continue;
        }
        if export.name != export_name && export.name != "*" {
            continue;
        }
        let resolved = resolver.resolve(source, file).map(|p| normalize_path(&p))?;
        let target_name = if imported == "*" {
            export_name
        } else {
            imported
        };
        if let Some(key) = resolve_component_key(
            project_root,
            shared,
            &resolved,
            target_name,
            component_keys,
            resolver,
            (visiting, demand),
        ) {
            return Some(key);
        }
    }
    None
}
