use std::collections::BTreeSet;

/// Referencing files for one declared name, using the same allowance and edge
/// emitter as the canonical language graph. A name the emitter does not link
/// stays absent.
pub(crate) fn proven_lang_reference_files(
    facts: &LangFactMap,
    declaration: &Path,
    name: &str,
    import_kind: EdgeKind,
    ref_kind: EdgeKind,
) -> BTreeSet<PathBuf> {
    let named = lang_name_files(facts, declaration, name);
    let mut edges = Vec::new();
    emit_lang_edges(
        facts,
        import_kind,
        ref_kind,
        &mut edges,
        &PathInterner::new(),
    );
    linked_sources(&named, &edges, declaration, ref_kind)
}

pub(crate) fn proven_swift_reference_files(
    facts: &crate::codebase::swift::SwiftFactMap,
    declaration: &Path,
    name: &str,
) -> BTreeSet<PathBuf> {
    let named = swift_name_files(facts, declaration, name);
    let mut edges = Vec::new();
    collect_swift_reference_edges(facts, &mut edges, &PathInterner::new());
    linked_sources(&named, &edges, declaration, EdgeKind::SwiftReference)
}

pub(crate) fn proven_dotnet_reference_files(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    declaration: &Path,
    name: &str,
) -> BTreeSet<PathBuf> {
    let named = dotnet_name_files(facts, declaration, name);
    let mut edges = Vec::new();
    collect_dotnet_reference_edges(facts, &mut edges, &PathInterner::new());
    linked_sources(&named, &edges, declaration, EdgeKind::DotnetReference)
}

fn lang_name_files(facts: &LangFactMap, declaration: &Path, name: &str) -> BTreeSet<PathBuf> {
    let mut sources = BTreeSet::new();
    let Some(target) = facts.files.get(declaration) else {
        return sources;
    };
    if !facts
        .declarations
        .get(name)
        .is_some_and(|paths| paths.contains(declaration))
    {
        return sources;
    }
    for file in facts.files.values() {
        if file.path == declaration || !file.references.iter().any(|item| item == name) {
            continue;
        }
        if reference_target_allowed(file, target, name) {
            sources.insert(file.path.clone());
        }
    }
    sources
}

fn swift_name_files(
    facts: &crate::codebase::swift::SwiftFactMap,
    declaration: &Path,
    name: &str,
) -> BTreeSet<PathBuf> {
    let mut sources = BTreeSet::new();
    if !facts
        .declarations
        .get(name)
        .is_some_and(|paths| paths.contains(declaration))
    {
        return sources;
    }
    for file in facts.files.values() {
        if file.path == declaration || !file.references.iter().any(|item| item == name) {
            continue;
        }
        let allowed = swift_allowed_roots(facts, file);
        if allowed.iter().any(|root| declaration.starts_with(root)) {
            sources.insert(file.path.clone());
        }
    }
    sources
}

fn dotnet_name_files(
    facts: &crate::codebase::dotnet::DotnetFactMap,
    declaration: &Path,
    name: &str,
) -> BTreeSet<PathBuf> {
    let mut sources = BTreeSet::new();
    if !facts
        .declarations
        .get(name)
        .is_some_and(|paths| paths.contains(declaration))
    {
        return sources;
    }
    for file in facts.files.values() {
        if file.path == declaration || !file.references.iter().any(|item| item == name) {
            continue;
        }
        let allowed = file
            .project
            .as_ref()
            .map(|project| dotnet_project_scope(facts, project))
            .unwrap_or_default();
        let targets = facts.declarations.get(name).cloned().unwrap_or_default();
        if scoped_dotnet_target_files(facts, &targets, &allowed).contains(declaration) {
            sources.insert(file.path.clone());
        }
    }
    sources
}

fn linked_sources(
    named: &BTreeSet<PathBuf>,
    edges: &[Edge],
    declaration: &Path,
    kind: EdgeKind,
) -> BTreeSet<PathBuf> {
    let mut linked = BTreeSet::new();
    for edge in edges {
        if edge.2 != kind || edge.1.as_file() != Some(declaration) {
            continue;
        }
        if let Some(path) = edge.0.as_file().filter(|path| named.contains(*path)) {
            linked.insert(path.to_path_buf());
        }
    }
    linked
}
