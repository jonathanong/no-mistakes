use super::bindings::Binding;
use super::ResourceFacts;
use std::collections::HashMap;

pub(super) struct ResourceVisitor<'a> {
    pub(super) source: &'a str,
    /// Each lexical scope maps a local name to either a supported resource
    /// binding or a deliberate shadow. Keeping shadows beside bindings avoids
    /// treating a parameter, local, or reassignment as the imported API.
    pub(super) bindings: Vec<HashMap<String, Option<Binding>>>,
    pub(super) function_stack: Vec<String>,
    /// Indexes in `bindings` that correspond to function scopes. `var` lives
    /// in the closest of these, whereas `let` and `const` live in the current
    /// lexical scope.
    pub(super) function_binding_scopes: Vec<usize>,
    /// Structural names for top-level object/class aggregates. These qualify
    /// callable members without making eager property initializers look like
    /// deferred function bodies.
    pub(super) aggregate_stack: Vec<String>,
    pub(super) anonymous_scopes: usize,
    pub(super) facts: ResourceFacts,
}

impl Default for ResourceVisitor<'_> {
    fn default() -> Self {
        Self {
            source: "",
            bindings: vec![HashMap::new()],
            function_stack: Vec::new(),
            function_binding_scopes: Vec::new(),
            aggregate_stack: Vec::new(),
            anonymous_scopes: 0,
            facts: ResourceFacts::default(),
        }
    }
}
