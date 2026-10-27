/// Members a type-only namespace import proves, or `None` when the namespace
/// escapes (`typeof ns`, no uses) and must remain an unknown selector.
fn type_namespace_members<'a>(
    file_facts: &'a crate::codebase::ts_source::facts::TsFileFacts,
    source: &str,
    local: &str,
) -> Option<&'a [String]> {
    file_facts
        .module_bindings
        .iter()
        .flat_map(|module| &module.imports)
        .filter(|import| import.specifier == source)
        .flat_map(|import| &import.bindings)
        .find(|binding| binding.local == local && binding.kind == "namespace")
        .and_then(|binding| binding.namespace_members.as_deref())
}

/// A named selector proven from a type position: the importer consumes
/// `name` of `target` exactly like `import type { name }`.
fn push_type_selector(
    inputs: &ExportEdgeInputs<'_>,
    target: &Path,
    name: &str,
    site: SelectorImportSite,
    exported: &FxHashSet<NodeId>,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    let symbol = NodeId::symbol_in(inputs.interner, target, name);
    if !exported.contains(&symbol) {
        selector_unknown_symbol(inputs, target, name, site, edges, sites);
    }
    edges.push((
        NodeId::file_in(inputs.interner, inputs.path),
        symbol,
        EdgeKind::SymbolImport,
    ));
}

/// `import("x").Name` and `import type * as ns` with only `ns.Name` uses name
/// their members, so they are symbol consumers. A bare `typeof import("x")`
/// carries no member and stays an unknown selector.
fn collect_type_import_selectors(
    inputs: &ExportEdgeInputs<'_>,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    exported: &FxHashSet<NodeId>,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) {
    let line_of = |start: u32| {
        file_facts.source.as_deref().map_or(1, |source| {
            crate::codebase::ts_source::byte_offset_to_line(source, start as usize) as usize
        })
    };
    for import in file_facts
        .module_bindings
        .iter()
        .flat_map(|module| &module.imports)
        .filter(|import| import.type_only && import.bindings.is_empty())
    {
        let line = line_of(import.span.start);
        let target = selector_target(inputs, &import.specifier).filter(|path| is_indexable(path));
        match (&import.qualifier, target) {
            (Some(name), Some(target)) => {
                let site = SelectorImportSite {
                    line,
                    specifier: import.specifier.clone(),
                    reason: "unresolved named selector",
                };
                push_type_selector(inputs, &target, name, site, exported, edges, sites);
            }
            _ => selector_unknown(
                inputs,
                &import.specifier,
                line,
                "type import selector",
                edges,
                sites,
            ),
        }
    }
    for import in &inputs.symbols.imports {
        let Some(members) = import
            .is_type_only
            .then(|| type_namespace_members(file_facts, &import.source, &import.local))
            .flatten()
            .filter(|_| import.imported == "*")
        else {
            continue;
        };
        let Some(target) = selector_target(inputs, &import.source).filter(|p| is_indexable(p))
        else {
            continue;
        };
        for name in members {
            let site = SelectorImportSite {
                line: import.line.max(1) as usize,
                specifier: import.source.clone(),
                reason: "unresolved named selector",
            };
            push_type_selector(inputs, &target, name, site, exported, edges, sites);
        }
    }
}
