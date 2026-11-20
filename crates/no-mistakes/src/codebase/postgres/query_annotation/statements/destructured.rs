use super::super::{expressions::expression, Expr, Step};
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};

// Destructuring a call result runs the call; inert placeholders keep each
// binding from being read as a builder. Ordinary lowering binds every call
// pattern as `const tmp = call()` would. A property default, a computed key,
// or a rest target that is not an identifier does not change which call runs.
// Those forms still run code the append proof cannot see, so that proof keeps
// them unsupported.
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
    if !matches!(
        call.get_inner_expression(),
        oxc_ast::ast::Expression::CallExpression(_)
    ) {
        return steps.push(Step::Unsupported);
    }
    if as_effect && !plain(&declaration.id) {
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
            // Primitive, not Unknown: an unlowered expression after the call is rejected.
            .map(|id| Step::Bind(id.name.to_string(), Expr::Primitive)),
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

// A supported summary records the call and drops the default, computed key, or
// non-identifier rest. Those still run after the callback, so the proof cannot
// attach. `plain` is the same gate the append lowering uses.
pub(in crate::codebase::postgres::query_annotation) fn hides_append_proof(
    value: &oxc_ast::ast::Function<'_>,
) -> bool {
    #[cfg(any(test, feature = "test-instrumentation"))]
    super::super::proof_walk::record();
    let mut finder = NonPlainCall { found: false };
    value
        .body
        .as_deref()
        .inspect(|body| finder.visit_function_body(body));
    finder.found
}

struct NonPlainCall {
    found: bool,
}

impl<'a> Visit<'a> for NonPlainCall {
    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        self.found |= call_initializer(declaration.init.as_ref()) && !plain(&declaration.id);
        walk::walk_variable_declarator(self, declaration);
    }
}

// Matches `bind`: await and type wrappers still leave a call expression.
fn call_initializer(init: Option<&Expression<'_>>) -> bool {
    init.is_some_and(|init| {
        let init = init.get_inner_expression();
        let call = match init {
            Expression::AwaitExpression(awaited) => awaited.argument.get_inner_expression(),
            other => other,
        };
        matches!(call, Expression::CallExpression(_))
    })
}
