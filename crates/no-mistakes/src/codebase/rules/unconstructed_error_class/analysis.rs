use super::ancestry::{bases, error_kinds, ClassKey, ErrorKind};
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{
    ClassDeclaration, DepGraph, KnownCallTargetRef, NodeId, ResolvedCallSite, ResolvedCallTarget,
};
use crate::fx::{fx_map, fx_set, FxHashMap};
use std::path::Path;

/// Declaration files describe code that lives outside the analyzed source, so
/// a class they declare is never a dead export of this project.
pub(super) fn is_declaration_file(path: &Path) -> bool {
    let name = path.to_string_lossy();
    [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|extension| name.ends_with(extension))
}

pub(super) fn unresolved_constructor(graph: &DepGraph, site: &ResolvedCallSite) -> bool {
    site.has_unknown_target()
        && !graph.constructor_proves_site(site)
        && (!matches!(site.target, ResolvedCallTarget::Unknown)
            || !graph.constructor_is_resolved(&site.file, site.offset))
}

/// Possible constructed classes, credited without duplicating source calls.
fn constructed_classes(site: &ResolvedCallSite) -> impl Iterator<Item = ClassKey<'_>> {
    site.targets()
        .filter_map(|candidate| match candidate.target {
            KnownCallTargetRef::RepositoryFunction { file, scope }
            | KnownCallTargetRef::ModuleExport {
                repository_target: Some((file, scope)),
                ..
            } => Some((file, scope)),
            _ => None,
        })
}

/// Exported error classes with no construction or subclass in non-test source.
///
/// A class declared with `declare` or in an ambient block is never reported. A
/// namespace member is uncertain when its namespace escapes:
/// when some use of the namespace is not a reference the graph resolves to a
/// class, whether the member is built is unknowable. Either kind still takes
/// part in ancestry analysis and known construction crediting.
///
/// `is_test` classifies files whose uses do not count, and `is_reported`
/// selects the declarations the caller wants findings for.
pub(super) struct UnconstructedClass<'a> {
    pub(super) class: &'a ClassDeclaration,
    pub(super) unknown_base: bool,
    pub(super) unknown_constructor: bool,
    pub(super) dynamic_ancestry: bool,
}

pub(super) fn unconstructed<'a>(
    graph: &'a DepGraph,
    is_test: impl Fn(&Path) -> bool,
    is_reported: impl Fn(&Path) -> bool,
) -> Vec<UnconstructedClass<'a>> {
    let classes = graph.class_declarations();
    let mut unused: FxHashMap<ClassKey<'a>, Vec<UnconstructedClass<'a>>> = fx_map();
    for (class, kind) in classes
        .iter()
        .zip(error_kinds(graph, classes))
        .filter(|(class, error)| *error != ErrorKind::Other && class.exported && !class.ambient)
        .filter(|(class, _)| {
            !is_test(&class.file) && is_reported(&class.file) && !is_declaration_file(&class.file)
        })
    {
        unused
            .entry((class.file.as_path(), class.scope.as_str()))
            .or_default()
            .push(UnconstructedClass {
                class,
                unknown_base: matches!(kind, ErrorKind::Unknown | ErrorKind::DynamicUnknown),
                dynamic_ancestry: kind == ErrorKind::DynamicUnknown,
                unknown_constructor: false,
            });
    }
    let mut uncertain_constructors = fx_set();
    for site in graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.invocation == InvocationKind::Construct)
    {
        if is_test(&site.file) {
            continue;
        }
        // An unknown call is not proof of construction or non-construction.
        // Retain its lexical association only as uncertainty, never credit.
        if unresolved_constructor(graph, site) {
            if let Some(scope) = site
                .caller
                .as_deref()
                .and_then(|caller| caller.split('/').next())
            {
                uncertain_constructors.insert((site.file.as_path(), scope));
            }
        }
        // Credit exact metadata before the legacy display-key fallback. A
        // known graph target must agree; unknown alternatives remain unknown.
        if let Some(
            target @ NodeId::Symbol {
                file,
                symbol,
                callable_id: Some(id),
            },
        ) = graph.constructor_class(&site.file, site.offset)
        {
            let proven = graph.constructor_proves_site(site)
                || matches!(site.target, ResolvedCallTarget::Unknown)
                || (!site.has_unknown_target() && site.target_nodes().any(|node| node == target));
            if proven {
                if let Some(classes) = unused.get_mut(&(file.as_ref(), symbol.as_ref())) {
                    classes.retain(|result| result.class.callable_id != *id);
                }
            }
        }
        for key in constructed_classes(site) {
            unused.remove(&key);
        }
    }
    for class in classes.iter().filter(|class| !is_test(&class.file)) {
        for base in bases(graph, class) {
            unused.remove(&base.key);
        }
    }
    let mut classes: Vec<_> = unused.into_values().flatten().collect();
    for result in &mut classes {
        result.unknown_constructor = uncertain_constructors
            .contains(&(result.class.file.as_path(), result.class.scope.as_str()));
    }
    classes.sort_by(|left, right| {
        (&left.class.file, left.class.line, &left.class.scope).cmp(&(
            &right.class.file,
            right.class.line,
            &right.class.scope,
        ))
    });
    classes
}
