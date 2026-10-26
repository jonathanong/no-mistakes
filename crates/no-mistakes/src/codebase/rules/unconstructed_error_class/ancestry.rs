use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::{ClassDeclaration, DepGraph, EdgeKind, NodeId};
use crate::fx::{fx_map, FxHashMap};
use std::path::Path;

/// A class identified by declaration file and display scope. Namespaces add no
/// scope component, so same-named classes in two namespaces of one file share a
/// key. Crediting works by key, so a construction or subclass of one credits
/// both: the rule stays quiet rather than guess. Ancestry is exact: an `Extends`
/// edge carries the base's parser id (see [`Base`]), so a subclass of `B.Base`
/// never becomes an error class through `A.Base`.
pub(super) type ClassKey<'a> = (&'a Path, &'a str);

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

/// A resolved class an `Extends` edge names: its file and display scope, and
/// its parser id when the edge carries one.
pub(super) struct Base<'a> {
    pub(super) key: ClassKey<'a>,
    id: Option<CallableId>,
}

/// The resolved classes `class` extends, read from its `Extends` edges.
/// Globals and unresolved external bases have no edge. Targets are compared by
/// file and scope rather than whole `NodeId`, which also carries a parser id
/// that two declarations sharing a display scope do not share.
pub(super) fn bases<'a>(
    graph: &'a DepGraph,
    class: &ClassDeclaration,
) -> impl Iterator<Item = Base<'a>> {
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

/// An unresolved base leaves error ancestry unknown. Resolved class
/// edges retain exact parser identities so same-named namespace classes do not mix.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ErrorKind {
    Other,
    Unknown,
    // Preserve dynamic ancestry through named descendants for policy routing.
    DynamicUnknown,
    Error,
}

pub(super) fn error_kinds(graph: &DepGraph, classes: &[ClassDeclaration]) -> Vec<ErrorKind> {
    let mut by_key: FxHashMap<ClassKey<'_>, Vec<usize>> = fx_map();
    let mut by_id: FxHashMap<(&Path, CallableId), usize> = fx_map();
    for (index, class) in classes.iter().enumerate() {
        by_key
            .entry((class.file.as_path(), class.scope.as_str()))
            .or_default()
            .push(index);
        by_id.insert((class.file.as_path(), class.callable_id), index);
    }
    // A closed ancestry cycle has no positive non-Error root proof.
    let mut flags = vec![ErrorKind::Unknown; classes.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for (index, class) in classes.iter().enumerate() {
            let mut parents = bases(graph, class).peekable();
            // ClassDeclaration records only classes with an extends base. No
            // edge or global means the declared base could not be analyzed.
            let mut kind = if class.global_base.as_deref().is_some_and(builtin_error) {
                ErrorKind::Error
            } else if class.dynamic_base {
                ErrorKind::DynamicUnknown
            } else if class.global_base.is_none() && parents.peek().is_none() {
                ErrorKind::Unknown
            } else {
                ErrorKind::Other
            };
            for base in parents {
                let parent_kind = match base.id {
                    Some(id) => by_id.get(&(base.key.0, id)).map(|parent| flags[*parent]),
                    None => by_key
                        .get(&base.key)
                        .and_then(|parents| parents.iter().map(|parent| flags[*parent]).max()),
                }
                .unwrap_or_else(|| {
                    if graph.class_is_root(base.key.0, base.key.1, base.id) {
                        ErrorKind::Other
                    } else {
                        ErrorKind::Unknown
                    }
                });
                kind = kind.max(parent_kind);
            }
            changed |= flags[index] != kind;
            flags[index] = kind;
        }
    }
    flags
}
