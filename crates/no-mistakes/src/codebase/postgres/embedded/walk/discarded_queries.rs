use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, Statement};

pub(super) fn record(statements: &[Statement<'_>], adjacent: &mut crate::fx::FxHashMap<u32, u32>) {
    let mut previous = None;
    for statement in statements {
        let next = awaited_call(statement);
        if let (Some(previous), Some(next)) = (previous, next) {
            adjacent.insert(previous, next);
        }
        previous = matches!(statement, Statement::ExpressionStatement(_))
            .then_some(next)
            .flatten();
    }
}

fn awaited_call(statement: &Statement<'_>) -> Option<u32> {
    let expression = match statement {
        Statement::ExpressionStatement(statement) => &statement.expression,
        Statement::ReturnStatement(statement) => statement.argument.as_ref()?,
        Statement::VariableDeclaration(statement) => {
            statement.declarations.first()?.init.as_ref()?
        }
        _ => return None,
    };
    let Expression::AwaitExpression(awaited) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&awaited.argument) else {
        return None;
    };
    let callee = match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional
                && matches!(
                    unwrap_ts_wrappers(&member.object),
                    Expression::Identifier(_)
                )
        }
        _ => false,
    };
    let mut arguments = call
        .arguments
        .iter()
        .map(|arg| arg.as_expression().map(unwrap_ts_wrappers));
    let static_sql = arguments.next()?.is_some_and(static_sql_argument);
    let safe_binds = match arguments.next() {
        None => true,
        Some(Some(Expression::ArrayExpression(array))) => array.elements.iter().all(|element| {
            matches!(
                element.as_expression().map(unwrap_ts_wrappers),
                Some(
                    Expression::Identifier(_)
                        | Expression::StringLiteral(_)
                        | Expression::NumericLiteral(_)
                        | Expression::BooleanLiteral(_)
                        | Expression::NullLiteral(_)
                )
            )
        }),
        _ => false,
    };
    (callee && !call.optional && static_sql && safe_binds && arguments.next().is_none())
        .then_some(call.span.start)
}

fn static_sql_argument(argument: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(argument) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::ConditionalExpression(choice) => {
            matches!(
                unwrap_ts_wrappers(&choice.test),
                Expression::Identifier(_) | Expression::BooleanLiteral(_)
            ) && static_sql_argument(&choice.consequent)
                && static_sql_argument(&choice.alternate)
        }
        _ => false,
    }
}
