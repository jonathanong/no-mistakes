use super::resolve::*;
use super::route_index::RegistryRoutes;
use crate::codebase::dependencies::extract::{ImportKind, ValueExpression};

/// New routes require a binding-to-edge association, never an unrelated export.
pub(crate) fn registry_route(
    graph: &DepGraph,
    routes: &RegistryRoutes<'_>,
    file: &Path,
    local: &str,
    members: &[String],
) -> Option<UniqueExport> {
    if let Some(binding) = routes.file(file)?.imports(routes).get(local).copied() {
        let producer = routes.target(graph, file, &binding.specifier, ImportKind::Static)?;
        if binding.kind == ImportedBindingKind::Namespace {
            return namespace_export(graph, routes, &producer, members);
        }
        let ts = routes.file(&producer)?;
        let namespace = ts
            .exports(routes)
            .namespaces
            .get(binding.imported.as_str())?;
        let source = routes.target(
            graph,
            &producer,
            namespace.specifier.as_deref()?,
            ImportKind::Static,
        )?;
        return namespace_export(graph, routes, &source, members);
    }
    let (producer, mut path) = require_binding(graph, routes, file, local)?;
    path.extend_from_slice(members);
    namespace_export(graph, routes, &producer, &path)
}
fn namespace_export(
    graph: &DepGraph,
    routes: &RegistryRoutes<'_>,
    file: &Path,
    members: &[String],
) -> Option<UniqueExport> {
    export_route(graph, routes, file, members, &mut crate::fx::fx_set())
}
fn export_route(
    graph: &DepGraph,
    routes: &RegistryRoutes<'_>,
    file: &Path,
    members: &[String],
    seen: &mut crate::fx::FxHashSet<(PathBuf, String)>,
) -> Option<UniqueExport> {
    let (exported, rest) = members.split_first()?;
    if !seen.insert((file.to_path_buf(), exported.clone())) {
        return None;
    }
    let ts = routes.file(file)?;
    let binding = ts.exports(routes).named.get(exported.as_str())?;
    if binding.specifier.is_none() {
        return Some(UniqueExport {
            producer: file.to_path_buf(),
            exported: exported.clone(),
        });
    }
    let next = routes.target(
        graph,
        file,
        binding.specifier.as_deref()?,
        ImportKind::Static,
    )?;
    if binding.local == "*" {
        export_route(graph, routes, &next, rest, seen)
    } else {
        export_route(
            graph,
            routes,
            &next,
            std::slice::from_ref(&binding.local),
            seen,
        )
    }
}
fn require_binding(
    graph: &DepGraph,
    routes: &RegistryRoutes<'_>,
    file: &Path,
    local: &str,
) -> Option<(PathBuf, Vec<String>)> {
    let ts = routes.file(file)?;
    let require = ts.requires(routes);
    let binding = require.bindings.get(local)?;
    let mut value = &binding.value;
    let mut members = Vec::new();
    while let ValueExpression::Member(inner, key) = value {
        members.push(key.clone());
        value = inner;
    }
    members.reverse();
    let ValueExpression::Call(start, _) = value else {
        return None;
    };
    let call = require.calls.get(start)?;
    let specifier = call.static_arg.as_deref()?;
    if !require.eager.contains(specifier) {
        return None;
    }
    Some((
        routes.target(graph, file, specifier, ImportKind::Require)?,
        members,
    ))
}
