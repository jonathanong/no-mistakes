use super::super::{expressions::expression, Expr, Step};

// Destructuring a call result runs the call; the unknown bindings keep the
// result from being read as a builder. Other initializers stay unsupported.
pub(super) fn bind(
    declaration: &oxc_ast::ast::VariableDeclarator<'_>,
    source: &str,
    steps: &mut Vec<Step>,
) {
    let call = match declaration
        .init
        .as_ref()
        .map(|init| init.get_inner_expression())
    {
        Some(oxc_ast::ast::Expression::AwaitExpression(value)) => &value.argument,
        Some(value) => value,
        None => return steps.push(Step::Unsupported),
    };
    if !matches!(
        call.get_inner_expression(),
        oxc_ast::ast::Expression::CallExpression(_)
    ) {
        return steps.push(Step::Unsupported);
    }
    let init = declaration.init.as_ref().expect("checked above");
    steps.push(Step::Effect(expression(init, source)));
    steps.extend(
        declaration
            .id
            .get_binding_identifiers()
            .into_iter()
            .map(|id| Step::Bind(id.name.to_string(), Expr::Unknown)),
    );
}
