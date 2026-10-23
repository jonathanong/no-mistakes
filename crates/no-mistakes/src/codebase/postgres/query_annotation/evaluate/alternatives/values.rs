use super::super::Value;
use super::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::FxHashSet;
mod prefixes;
mod progress;
pub(super) use progress::Walk;

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

pub(super) fn apply_taint(scopes: &mut [Scope], changed: &FxHashSet<u64>) -> bool {
    if changed.is_empty() {
        return false;
    }
    // Aliased frames share one bindings snapshot. Taint that snapshot once and
    // install it everywhere, instead of cloning every alias on the first write.
    let mut tainted = crate::fx::FxHashMap::<usize, Scope>::default();
    let mut wrote = false;
    for scope in scopes {
        if !scope.contains_builders(changed) {
            continue;
        }
        let shared = scope.snapshot_addr();
        if let Some(done) = tainted.get(&shared) {
            *scope = done.clone();
            continue;
        }
        for value in scope.values_mut() {
            if let Value::Prefix(_, _, Some(id)) = value {
                if changed.contains(id) {
                    *value = Value::Unknown;
                    wrote = true;
                }
            }
        }
        tainted.insert(shared, scope.clone());
    }
    wrote
}

#[cfg(test)]
mod taint_tests {
    use super::apply_taint;
    use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
    use crate::fx::{FxHashMap, FxHashSet};

    #[test]
    fn shared_frames_keep_one_tainted_snapshot() {
        let shared = Scope::from(FxHashMap::from_iter([
            (
                "sql".into(),
                Value::Prefix("select 1".into(), true, Some(7)),
            ),
            (
                "other".into(),
                Value::Prefix("select 2".into(), true, Some(8)),
            ),
        ]));
        let mut scopes = vec![shared.clone(), shared.clone(), shared];
        let mut changed = FxHashSet::default();
        changed.insert(7);
        apply_taint(&mut scopes, &changed);
        assert!(scopes[0].same_bindings(&scopes[1]));
        assert!(scopes[1].same_bindings(&scopes[2]));
        assert!(matches!(scopes[0].get("sql"), Some(Value::Unknown)));
        assert!(matches!(
            scopes[2].get("other"),
            Some(Value::Prefix(_, _, Some(8)))
        ));
    }
}
