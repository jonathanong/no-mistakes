use crate::react_traits::analyze::components::ComponentDef;
use crate::react_traits::traits::{memo, props, suspense};
use oxc_ast::ast::Program;

use super::FileTraitHits;

pub(super) fn fill_declaration_traits(
    program: &Program<'_>,
    defs: &[ComponentDef],
    hits: &mut FileTraitHits,
) {
    for (index, def) in defs.iter().enumerate() {
        hits.has_props[index] = props::has_function_params(program, def.span);
        hits.uses_memo[index] |= memo::is_wrapped_in_memo(program, def);
        hits.uses_suspense_jsx[index] |= suspense::is_component_direct_lazy(program, def.span);
    }
}
