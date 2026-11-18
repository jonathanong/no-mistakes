use super::super::{expressions::expression, Expr, Step};
use oxc_ast::ast::BindingPattern;

// Destructuring a call result runs the call; the unknown bindings keep the
// result from being read as a builder. Defaults, computed keys and non-plain
// rest targets run code the summary cannot see, so they stay unsupported.
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
    if !plain(&declaration.id) {
        return steps.push(Step::Unsupported);
    }
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

fn plain(pattern: &BindingPattern<'_>) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => true,
        BindingPattern::ObjectPattern(object) => {
            object
                .properties
                .iter()
                .all(|property| !property.computed && plain(&property.value))
                && object
                    .rest
                    .as_ref()
                    .is_none_or(|rest| plain_target(&rest.argument))
        }
        BindingPattern::ArrayPattern(array) => {
            array.elements.iter().flatten().all(plain)
                && array
                    .rest
                    .as_ref()
                    .is_none_or(|rest| plain_target(&rest.argument))
        }
        BindingPattern::AssignmentPattern(_) => false,
    }
}

fn plain_target(pattern: &BindingPattern<'_>) -> bool {
    matches!(pattern, BindingPattern::BindingIdentifier(_))
}
