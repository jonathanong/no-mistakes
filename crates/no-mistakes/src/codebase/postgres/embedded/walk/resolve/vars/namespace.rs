use super::ScopeVisitor;
use oxc_ast::ast::{BindingPattern, VariableDeclarator};

pub(in crate::codebase::postgres::embedded::walk) fn bind_query_property(
    declarator: &VariableDeclarator<'_>,
    is_const: bool,
    visitor: &mut ScopeVisitor<'_>,
) {
    let (Some(init), BindingPattern::ObjectPattern(object)) = (&declarator.init, &declarator.id)
    else {
        return;
    };
    if !is_const || !visitor.namespace_expression(init) || !visitor.namespace_query_valid() {
        return;
    }
    for property in &object.properties {
        if property.key.static_name().as_deref() != Some("query") {
            continue;
        }
        let BindingPattern::BindingIdentifier(local) = &property.value else {
            continue;
        };
        if let Some(binding) = visitor
            .current_scope()
            .and_then(|scope| scope.get_mut(local.name.as_str()))
        {
            binding.executor_alias = Some(true);
        }
    }
}
