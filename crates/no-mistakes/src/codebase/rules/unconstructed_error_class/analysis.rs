use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::{
    ClassDeclaration, DepGraph, EdgeKind, NodeId, ResolvedCallSite, ResolvedCallTarget,
};
use crate::fx::{fx_map, FxHashMap};
use std::path::Path;

/// A class identified by declaration file and display scope. Namespaces add no
/// scope component, so same-named classes in two namespaces of one file share a
/// key. Crediting works by key, so a construction or subclass of one credits
/// both: the rule stays quiet rather than guess. Ancestry is exact: an `Extends`
/// edge carries the base's parser id (see [`Base`]), so a subclass of `B.Base`
/// never becomes an error class through `A.Base`.
type ClassKey<'a> = (&'a Path, &'a str);

const BUILTIN_ERRORS: &[&str] = &[
    "Error",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
    "AggregateError",
    "SuppressedError",
];
const GLOBAL_OBJECTS: &[&str] = &["globalThis.", "window.", "self.", "global."];

fn builtin_error(name: &str) -> bool {
    let name = GLOBAL_OBJECTS
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .unwrap_or(name);
    BUILTIN_ERRORS.contains(&name)
}

/// Declaration files describe code that lives outside the analyzed source, so
/// a class they declare is never a dead export of this project.
pub(super) fn is_declaration_file(path: &Path) -> bool {
    let name = path.to_string_lossy();
    [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|extension| name.ends_with(extension))
}

/// The repository class a resolved target names, when it names one.
fn class_key(target: &ResolvedCallTarget) -> Option<ClassKey<'_>> {
    match target {
        ResolvedCallTarget::RepositoryFunction { file, scope }
        | ResolvedCallTarget::ModuleExport {
            repository_target: Some((file, scope)),
            ..
        } => Some((file.as_path(), scope.as_str())),
        _ => None,
    }
}

/// The class a `new` expression builds. An unresolved `new` inside a class
/// member, such as `new this()` in a static factory, is credited to that class:
/// the graph cannot say what it builds, and the rule prefers silence to a false
/// finding.
fn constructed_class(site: &ResolvedCallSite) -> Option<ClassKey<'_>> {
    match &site.target {
        ResolvedCallTarget::Unknown => site
            .caller
            .as_deref()
            .and_then(|caller| caller.split('/').next())
            .map(|scope| (site.file.as_path(), scope)),
        target => class_key(target),
    }
}

/// A repository class an `Extends` edge names: its file and display scope, and
/// its parser id when the edge carries one.
struct Base<'a> {
    key: ClassKey<'a>,
    id: Option<CallableId>,
}

/// The repository classes `class` extends, read from its `Extends` edges. A
/// global or external-package base has no edge. Targets are compared by file
/// and scope rather than whole `NodeId`, which also carries a parser id that
/// two declarations sharing a display scope do not share.
fn bases<'a>(graph: &'a DepGraph, class: &ClassDeclaration) -> impl Iterator<Item = Base<'a>> {
    graph
        .dependencies_of_node(&class.node())
        .into_iter()
        .flatten()
        .filter(|(_, kind)| *kind == EdgeKind::Extends)
        .filter_map(|(target, _)| match target {
            NodeId::Symbol {
                file,
                symbol,
                callable_id,
            } => Some(Base {
                key: (file.as_ref(), &**symbol),
                id: *callable_id,
            }),
            _ => None,
        })
}

/// Marks every class whose `extends` chain reaches a built-in error. A base
/// from an external package is not a root: its ancestry is unknown here.
fn error_flags(graph: &DepGraph, classes: &[ClassDeclaration]) -> Vec<bool> {
    let mut by_key: FxHashMap<ClassKey<'_>, Vec<usize>> = fx_map();
    let mut by_id: FxHashMap<(&Path, CallableId), usize> = fx_map();
    for (index, class) in classes.iter().enumerate() {
        by_key
            .entry((class.file.as_path(), class.scope.as_str()))
            .or_default()
            .push(index);
        by_id.insert((class.file.as_path(), class.callable_id), index);
    }
    let mut flags = vec![false; classes.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for (index, class) in classes.iter().enumerate() {
            if flags[index] {
                continue;
            }
            // A base's exact id picks one declaration out of those sharing its
            // scope; a class with no `extends` has no declaration here, so an
            // id with none is no error class. Without a known id, any
            // declaration of the scope will do.
            let error = class.global_base.as_deref().is_some_and(builtin_error)
                || bases(graph, class).any(|base| match base.id {
                    Some(id) => by_id
                        .get(&(base.key.0, id))
                        .is_some_and(|parent| flags[*parent]),
                    None => by_key
                        .get(&base.key)
                        .is_some_and(|parents| parents.iter().any(|parent| flags[*parent])),
                });
            flags[index] = error;
            changed |= error;
        }
    }
    flags
}

/// Exported error classes with no construction or subclass in non-test source.
///
/// A class declared with `declare` or in an ambient block is never reported. A
/// namespace member is reported like any class, unless its namespace escapes:
/// when some use of the namespace is not a reference the graph resolves to a
/// class, whether the member is built is unknowable. Either kind still takes
/// part in `error_flags` and crediting.
///
/// `is_test` classifies files whose uses do not count, and `is_reported`
/// selects the declarations the caller wants findings for.
pub(super) fn unconstructed<'a>(
    graph: &'a DepGraph,
    is_test: impl Fn(&Path) -> bool,
    is_reported: impl Fn(&Path) -> bool,
) -> Vec<&'a ClassDeclaration> {
    let classes = graph.class_declarations();
    let mut unused: FxHashMap<ClassKey<'a>, Vec<&'a ClassDeclaration>> = fx_map();
    for (class, _) in classes
        .iter()
        .zip(error_flags(graph, classes))
        .filter(|(class, error)| {
            *error && class.exported && !class.ambient && !class.namespace_escaped
        })
        .filter(|(class, _)| {
            !is_test(&class.file) && is_reported(&class.file) && !is_declaration_file(&class.file)
        })
    {
        unused
            .entry((class.file.as_path(), class.scope.as_str()))
            .or_default()
            .push(class);
    }
    let mut credit = |key: ClassKey<'a>, file: &Path| {
        if !is_test(file) {
            unused.remove(&key);
        }
    };
    for site in graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.invocation == InvocationKind::Construct)
    {
        if let Some(key) = constructed_class(site) {
            credit(key, &site.file);
        }
    }
    for class in classes {
        for base in bases(graph, class) {
            credit(base.key, &class.file);
        }
    }
    let mut classes: Vec<_> = unused.into_values().flatten().collect();
    classes.sort_by(|left, right| {
        (&left.file, left.line, &left.scope).cmp(&(&right.file, right.line, &right.scope))
    });
    classes
}
