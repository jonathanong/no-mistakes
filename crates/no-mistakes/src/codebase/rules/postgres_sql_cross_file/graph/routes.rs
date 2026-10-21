use super::resolve::*;
use crate::codebase::dependencies::extract::{CallTargetIdentity, ImportKind, ValueExpression};

/// New routes require a binding-to-edge association, never an unrelated export.
pub(crate) fn registry_route(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    local: &str,
    members: &[String],
) -> Option<UniqueExport> {
    if let Some(binding) = imported(facts, file, local) {
        let producer = target(graph, facts, file, &binding.specifier, ImportKind::Static)?;
        if binding.kind == ImportedBindingKind::Namespace {
            return namespace_export(graph, facts, &producer, members);
        }
        let ts = facts.get_ts_facts(&producer)?;
        let namespace = ts
            .exported_bindings
            .iter()
            .find(|export| export.exported == binding.imported && export.local == "*")?;
        let source = target(
            graph,
            facts,
            &producer,
            namespace.specifier.as_deref()?,
            ImportKind::Static,
        )?;
        return namespace_export(graph, facts, &source, members);
    }
    let (producer, mut path) = require_binding(graph, facts, file, local)?;
    path.extend_from_slice(members);
    namespace_export(graph, facts, &producer, &path)
}
fn namespace_export(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    members: &[String],
) -> Option<UniqueExport> {
    export_route(graph, facts, file, members, &mut crate::fx::fx_set())
}
fn export_route(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    members: &[String],
    seen: &mut crate::fx::FxHashSet<(PathBuf, String)>,
) -> Option<UniqueExport> {
    let (exported, rest) = members.split_first()?;
    if !seen.insert((file.to_path_buf(), exported.clone())) {
        return None;
    }
    let ts = facts.get_ts_facts(file)?;
    let binding = ts
        .exported_bindings
        .iter()
        .find(|binding| binding.exported == *exported)?;
    if binding.specifier.is_none() {
        return Some(UniqueExport {
            producer: file.to_path_buf(),
            exported: exported.clone(),
        });
    }
    let next = target(
        graph,
        facts,
        file,
        binding.specifier.as_deref()?,
        ImportKind::Static,
    )?;
    if binding.local == "*" {
        export_route(graph, facts, &next, rest, seen)
    } else {
        export_route(
            graph,
            facts,
            &next,
            std::slice::from_ref(&binding.local),
            seen,
        )
    }
}
fn require_binding(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    local: &str,
) -> Option<(PathBuf, Vec<String>)> {
    let ts = facts.get_ts_facts(file)?;
    let binding = ts.value_flow.bindings.iter().find(|binding| {
        binding.owner.is_none() && binding.scope == 0 && binding.name == local && binding.immutable
    })?;
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
    let call = ts.function_calls.iter().find(|call| {
        call.offset == *start
            && call.callee == "require"
            && call.target_identity == CallTargetIdentity::Global
            && call.callee_binding_scope.is_none()
    })?;
    let specifier = call.static_arg.as_deref()?;
    if !ts
        .eager_require_specifiers
        .iter()
        .any(|value| value == specifier)
    {
        return None;
    }
    Some((
        target(graph, facts, file, specifier, ImportKind::Require)?,
        members,
    ))
}
/// Remove proven external module nodes, then require exactly one remaining
/// specifier and one file edge. This associates the actual literal binding.
fn target(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    specifier: &str,
    kind: ImportKind,
) -> Option<PathBuf> {
    let edge = if kind == ImportKind::Require {
        EdgeKind::Require
    } else {
        EdgeKind::Import
    };
    let neighbors = graph.dependencies_of_node(&NodeId::file(file))?;
    let external: Vec<_> = neighbors
        .iter()
        .filter_map(|(target, actual)| match target {
            NodeId::Module(name) if *actual == edge => Some(&**name),
            _ => None,
        })
        .collect();
    let ts = facts.get_ts_facts(file)?;
    let mut specifiers = Vec::new();
    for import in ts.imports.iter().filter(|import| import.kind == kind) {
        if import.computed {
            return None;
        }
        if !external.contains(&import.specifier.as_str()) {
            specifiers.push(import.specifier.as_str());
        }
    }
    specifiers.sort();
    specifiers.dedup();
    if specifiers != [specifier] {
        return None;
    }
    let mut files = neighbors
        .iter()
        .filter_map(|(target, actual)| (*actual == edge).then(|| target.as_file()).flatten())
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    let [file] = files.as_slice() else {
        return None;
    };
    Some(file.to_path_buf())
}
