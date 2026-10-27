use super::super::Value;
use super::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::FxHashSet;
mod prefixes;
mod progress;
pub(super) use progress::Walk;

pub(super) fn unchanged_without_arguments(before: &Scope, after: &Scope) -> bool {
    // Shared bindings fix every immutable prefix, but not mutable arena slots.
    // Borrow only a warm summary: cold scopes retain their original walk.
    before.same_bindings(after)
        && before
            .cached_references()
            .is_some_and(|edges| edges.arguments.is_empty())
}

pub(super) fn changes(
    before: &Value,
    after: &Value,
    before_arena: Arena<'_>,
    after_arena: Arena<'_>,
    changed: &mut FxHashSet<u64>,
    definite: &FxHashSet<(u64, usize)>,
    work: &mut Option<Walk>,
) {
    if let Some(work) = work.as_mut() {
        work.comparison();
    }
    if work.is_some() {
        compare::<true>(
            before,
            after,
            before_arena,
            after_arena,
            changed,
            definite,
            work,
        );
    } else {
        compare::<false>(
            before,
            after,
            before_arena,
            after_arena,
            changed,
            definite,
            work,
        );
    }
}

fn compare<const OBSERVED: bool>(
    before: &Value,
    after: &Value,
    before_arena: Arena<'_>,
    after_arena: Arena<'_>,
    changed: &mut FxHashSet<u64>,
    definite: &FxHashSet<(u64, usize)>,
    work: &mut Option<Walk>,
) {
    let previous = prefixes::collect::<OBSERVED>(before, before_arena, definite, work);
    let current = prefixes::collect::<OBSERVED>(after, after_arena, definite, work);
    if let Some(work) = work.as_mut() {
        work.complete();
    }
    for (id, (text, complete)) in previous {
        // A reference disappearing from this binding is a rebind, not a mutation.
        // Opaque mutation of the referenced identity is tracked independently.
        let Some((other, done)) = current.get(&id) else {
            continue;
        };
        let unchanged = (text == *other && complete == *done)
            || (text.trim_start().starts_with("/*") && other.trim_start().starts_with("/*"));
        if !unchanged {
            changed.insert(id);
        }
    }
}

pub(super) fn apply_taint(scopes: &mut [Scope], changed: &FxHashSet<u64>) {
    if changed.is_empty() {
        return;
    }
    for scope in scopes {
        if !scope.contains_builders(changed) {
            continue;
        }
        for value in scope.values_mut() {
            if let Value::Prefix(_, _, Some(id)) = value {
                if changed.contains(id) {
                    *value = Value::Unknown;
                }
            }
        }
    }
}
