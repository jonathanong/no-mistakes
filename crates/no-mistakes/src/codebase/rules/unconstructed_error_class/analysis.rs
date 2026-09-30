use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{
    DepGraph, ResolvedCallSite, ResolvedCallTarget, ResolvedClassBase,
};
use crate::fx::{fx_map, FxHashMap};
use std::path::Path;

/// A class identified by declaration file and display scope. Exported classes
/// are top level, so the pair is unique in practice; a collision only makes the
/// rule quieter, never noisier.
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

/// Marks every class whose `extends` chain reaches a built-in error. A base
/// from an external package is not a root: its ancestry is unknown here.
fn error_flags(bases: &[ResolvedClassBase]) -> Vec<bool> {
    let mut by_key: FxHashMap<ClassKey<'_>, Vec<usize>> = fx_map();
    for (index, class) in bases.iter().enumerate() {
        by_key
            .entry((class.file.as_path(), class.class_scope.as_str()))
            .or_default()
            .push(index);
    }
    let mut flags = vec![false; bases.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for (index, class) in bases.iter().enumerate() {
            if flags[index] {
                continue;
            }
            let error = match &class.base {
                ResolvedCallTarget::Global { name } => builtin_error(name),
                other => class_key(other)
                    .and_then(|key| by_key.get(&key))
                    .is_some_and(|parents| parents.iter().any(|parent| flags[*parent])),
            };
            flags[index] = error;
            changed |= error;
        }
    }
    flags
}

/// Exported error classes with no construction or subclass in non-test source.
///
/// `is_test` classifies files whose uses do not count, and `is_reported`
/// selects the declarations the caller wants findings for.
pub(super) fn unconstructed<'a>(
    graph: &'a DepGraph,
    is_test: impl Fn(&Path) -> bool,
    is_reported: impl Fn(&Path) -> bool,
) -> Vec<&'a ResolvedClassBase> {
    let bases = graph.resolved_class_bases();
    let mut unused: FxHashMap<ClassKey<'a>, Vec<&'a ResolvedClassBase>> = fx_map();
    for (class, _) in bases
        .iter()
        .zip(error_flags(bases))
        .filter(|(class, error)| *error && class.exported)
        .filter(|(class, _)| !is_test(&class.file) && is_reported(&class.file))
    {
        unused
            .entry((class.file.as_path(), class.class_scope.as_str()))
            .or_default()
            .push(class);
    }
    if unused.is_empty() {
        return Vec::new();
    }
    let constructed = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.invocation == InvocationKind::Construct)
        .map(|site| (constructed_class(site), site.file.as_path()));
    let subclassed = bases
        .iter()
        .map(|class| (class_key(&class.base), class.file.as_path()));
    for (key, file) in constructed.chain(subclassed) {
        if let Some(key) = key.filter(|key| unused.contains_key(key)) {
            if !is_test(file) {
                unused.remove(&key);
            }
        }
    }
    let mut classes: Vec<_> = unused.into_values().flatten().collect();
    classes.sort_by(|left, right| {
        (&left.file, left.line, &left.class_scope).cmp(&(
            &right.file,
            right.line,
            &right.class_scope,
        ))
    });
    classes
}
