use super::*;

#[path = "array_element_values.rs"]
mod array_element_values;
#[path = "escaped_array_bindings.rs"]
mod escaped_array_bindings;
#[path = "escaped_array_entries.rs"]
mod escaped_array_entries;
#[path = "escaped_call_arguments.rs"]
mod escaped_call_arguments;
#[path = "escaped_class_aliases.rs"]
mod escaped_class_aliases;
#[path = "escaped_class_lookup.rs"]
mod escaped_class_lookup;
#[path = "escaped_class_methods.rs"]
mod escaped_class_methods;
#[path = "escaped_guarded_arrays.rs"]
mod escaped_guarded_arrays;
#[path = "escaped_higher_order.rs"]
mod escaped_higher_order;
#[path = "escaped_origin_absorb.rs"]
mod escaped_origin_absorb;
#[path = "escaped_parameter_identity.rs"]
mod escaped_parameter_identity;
#[path = "escaped_parameters.rs"]
mod escaped_parameters;
#[path = "escaped_possible_arguments.rs"]
mod escaped_possible_arguments;
#[path = "escaped_possible_mutations.rs"]
mod escaped_possible_mutations;
#[path = "escaped_spread_arguments.rs"]
mod escaped_spread_arguments;

pub(super) use array_element_values::value as array_element_value;
pub(super) use escaped_array_entries::{array_elements, assign_sources, copied_member};
pub(super) use escaped_origin_absorb::{absorb, absorb_possible};

type StoredArray = (
    usize,
    String,
    u32,
    std::sync::Arc<FxHashMap<String, ValueExpression>>,
    bool,
);

pub(super) struct UncertainSpread {
    pub(super) raw: usize,
    pub(super) shift: usize,
}

#[derive(Default)]
pub(in super::super) struct Side {
    /// `const api = new Api()` — write origins only, not a callable binding.
    /// Keyed by binding scope and name. Dotted keys are instance fields.
    instances: FxHashMap<(usize, String), CallableId>,
    pub(in super::super) base_scopes: FxHashMap<CallableId, usize>,
    /// Arrow callables. `arguments` belongs to the enclosing function.
    arrows: FxHashSet<CallableId>,
    /// Instance field `(class, name)` initialized with an arrow or function.
    fields: FxHashMap<(CallableId, String), CallableId>,
    /// `return new Class()` recorded for the returning function.
    returns: FxHashMap<CallableId, CallableId>,
    /// `const r = instance.method`.
    method_aliases: FxHashMap<(usize, String), CallableId>,
    /// `const a = arguments`.
    argument_aliases: FxHashSet<(usize, String)>,
    /// Array values from `let` bindings and later array assignments.
    arrays: Vec<StoredArray>,
    guarded_arrays: bool,
    optional_pushes: FxHashSet<u32>,
    pushes: FxHashMap<usize, FxHashMap<String, Vec<usize>>>,
    pending_possible: Vec<escaped_possible_arguments::Pending>,
    stable_bindings: FxHashMap<usize, FxHashSet<String>>,
    parameter_epochs: FxHashMap<usize, FxHashMap<String, escaped_parameter_identity::Epochs>>,
    pub(in super::super) parameter_completion: Option<(u32, u32)>,
    guarded_summaries: FxHashMap<usize, FxHashMap<String, escaped_guarded_arrays::Summary>>,
    guarded_escaped: FxHashMap<(u32, u32), FxHashMap<usize, Vec<ValueExpression>>>,
    guarded_open: FxHashMap<(u32, u32), (usize, Vec<ValueExpression>)>,
    /// Call span `(start, end)`, method, and the raw argument values. Spread and
    /// conditional shapes are also stored in the escaped maps under that same span.
    methods: Vec<(u32, u32, CallableId, Vec<ValueExpression>, bool)>,
    /// Conditional and spread arguments. Keyed by call span `(start, end)`, then
    /// parameter index.
    escaped: FxHashMap<(u32, u32), FxHashMap<usize, Vec<ValueExpression>>>,
    /// Values that can occupy every parameter at or after the index, because a
    /// spread ahead of them had no known elements.
    open: FxHashMap<(u32, u32), (usize, Vec<ValueExpression>)>,
    pub(super) uncertain_spreads: FxHashMap<(u32, u32), UncertainSpread>,
    pub(super) reassigned: Vec<super::parameter_call_origins::Reassignment>,
    possible_escaped: FxHashMap<(u32, u32), FxHashMap<usize, Vec<ValueExpression>>>,
    possible_open: FxHashMap<(u32, u32), (usize, Vec<ValueExpression>)>,
    pub(super) projecting_possible: bool,
}
