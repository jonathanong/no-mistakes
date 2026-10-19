use super::*;

#[path = "escaped_array_entries.rs"]
mod escaped_array_entries;
#[path = "escaped_call_arguments.rs"]
mod escaped_call_arguments;
#[path = "escaped_class_methods.rs"]
mod escaped_class_methods;
#[path = "escaped_origin_absorb.rs"]
mod escaped_origin_absorb;
#[path = "escaped_parameters.rs"]
mod escaped_parameters;

pub(super) use escaped_array_entries::{array_elements, assign_sources, copied_member};
pub(super) use escaped_origin_absorb::absorb;

#[derive(Default)]
pub(in super::super) struct Side {
    /// `const api = new Api()` — write origins only, not a callable binding.
    /// Keyed by binding scope and name.
    instances: FxHashMap<(usize, String), CallableId>,
    /// Call span `(start, end)`, method, and the raw argument values. Spread and
    /// conditional shapes are also stored in the escaped maps under that same span.
    methods: Vec<(u32, u32, CallableId, Vec<ValueExpression>)>,
    /// Conditional and spread arguments. Keyed by call span `(start, end)`, then
    /// parameter index.
    escaped: FxHashMap<(u32, u32), FxHashMap<usize, Vec<ValueExpression>>>,
    /// Values that can occupy every parameter at or after the index, because a
    /// spread ahead of them had no known elements.
    open: FxHashMap<(u32, u32), (usize, Vec<ValueExpression>)>,
    reassigned: Vec<(usize, String, ValueExpression)>,
}
