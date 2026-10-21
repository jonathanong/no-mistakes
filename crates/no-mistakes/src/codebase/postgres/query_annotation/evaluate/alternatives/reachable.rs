use super::super::temporaries::TransientRoot;
use super::super::{Environment, Value};
use super::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::{CapturedBindings, Scope};
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct RootSeeds<'a> {
    pub environments: &'a [Environment],
    pub arguments: &'a [u64],
    pub transient: &'a [TransientRoot],
}

#[derive(Default)]
pub(super) struct Reachable {
    pub environments: FxHashSet<Environment>,
    pub arguments: FxHashSet<u64>,
    pub identities: FxHashSet<u64>,
    pending_env: Vec<Environment>,
    pending_args: Vec<u64>,
    visited_values: FxHashSet<*const ()>,
}

#[cfg(not(test))]
#[path = "reachable/scan_runtime.rs"]
#[macro_use]
mod scan_hooks;
#[cfg(test)]
#[path = "reachable/scan_tests.rs"]
#[macro_use]
pub(super) mod scan_hooks;

impl Reachable {
    fn value(&mut self, value: &Value) {
        record_raw_value!();
        match value {
            Value::Function(_, _, env) => self.pending_env.push(*env),
            Value::Arguments(id) | Value::Array(id) => self.pending_args.push(*id),
            Value::Prefix(_, _, Some(id)) => {
                self.identities.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.value(value),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values)
                // Functions carry environments without being builder/argument
                // references. Both immutable summaries must be empty to skip.
                if (values.contains_reference() || values.contains_environment())
                    && self.visited_values.insert(values.identity()) =>
            {
                for value in values {
                    self.value(value);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "reachable/tests.rs"]
mod tests;

mod walk;

pub(super) fn collect_from_roots(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) -> Reachable {
    walk::collect::<false>(scopes, roots, returned, arena, mapped, captured)
}

pub(super) fn collect_warm_from_roots(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) -> Reachable {
    walk::collect::<true>(scopes, roots, returned, arena, mapped, captured)
}

mod prefix;
pub(super) use prefix::Prefix;
mod cached;
pub(super) use cached::collect_cached;

#[cfg(test)]
#[path = "reachable/cached_tests.rs"]
mod cached_tests;

#[cfg(test)]
#[path = "reachable/cached_guards_tests.rs"]
mod cached_guards_tests;
