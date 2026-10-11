use super::ScopeVisitor;
use oxc_ast::ast::TemplateLiteral;

pub(super) fn mutates_captured_fragment(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
) -> bool {
    let last = template.expressions.len().saturating_sub(1);
    let mut captured_builder = false;
    template
        .expressions
        .iter()
        .enumerate()
        .any(|(index, expression)| {
            // The tag runs after all expressions. A final mutation can alter
            // an earlier object reference, but not an already-copied primitive.
            if (index < last || captured_builder) && visitor.expression_mutates_builder(expression)
            {
                return true;
            }
            captured_builder |= visitor.builder_result_names(expression).iter().any(|name| {
                visitor.lookup(name).is_some_and(|binding| {
                    binding.builder_identity.is_some()
                        || binding
                            .variants
                            .as_ref()
                            .is_some_and(|values| values.iter().any(|value| value.fragment))
                })
            });
            false
        })
}
