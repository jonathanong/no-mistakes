use super::super::{expressions::expression, Expr, Step};
use oxc_ast::ast::BindingPattern;

// Destructuring a call result runs the call; the unknown bindings keep the
// result from being read as a builder. The append proof evaluates it as an
// effect; ordinary lowering binds it as `const tmp = call()` would, so both
// forms resolve and report identically. Defaults, computed keys and non-plain
// rest targets run code the summary cannot see, so they stay unsupported.
pub(super) fn bind(
    declaration: &oxc_ast::ast::VariableDeclarator<'_>,
    source: &str,
    as_effect: bool,
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
    let init = expression(init, source);
    steps.push(if as_effect {
        Step::Effect(init)
    } else {
        Step::Bind(RESULT.to_string(), init)
    });
    steps.extend(
        declaration
            .id
            .get_binding_identifiers()
            .into_iter()
            .map(|id| Step::Bind(id.name.to_string(), Expr::Unknown)),
    );
}

// A literal leading annotation proves the statement for every caller, so the
// lexical facts already answer it; evaluating the call only adds work.
pub(in crate::codebase::postgres::query_annotation) fn literal_annotated_call(init: &Expr) -> bool {
    let init = match init {
        Expr::Await(inner) => inner,
        other => other,
    };
    let Expr::Call { args, .. } = init else {
        return false;
    };
    args.first().is_some_and(annotated_head)
}

fn annotated_head(expr: &Expr) -> bool {
    use crate::codebase::postgres::annotation::has_leading_query_annotation as literal_head;
    match expr {
        Expr::Text(text) => literal_head(text),
        Expr::Template(parts) | Expr::Tagged(_, parts, _) => {
            matches!(parts.first(), Some(Expr::Text(text)) if literal_head(text))
        }
        Expr::Append(base, _) => annotated_head(base),
        _ => false,
    }
}

pub(in crate::codebase::postgres::query_annotation) const RESULT: &str = "<destructured result>";

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
