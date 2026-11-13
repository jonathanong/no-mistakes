use super::*;

pub(super) fn returning_guard(
    statement: &oxc_ast::ast::IfStatement<'_>,
    name: &str,
    source: Option<&str>,
) -> bool {
    statement.alternate.is_none()
        && read_only(&statement.test, name, source)
        && returning_branch(&statement.consequent, name, source)
}

fn returning_branch(statement: &Statement<'_>, name: &str, source: Option<&str>) -> bool {
    match statement {
        Statement::ReturnStatement(statement) => statement
            .argument
            .as_ref()
            .is_none_or(|value| read_only(value, name, source)),
        Statement::BlockStatement(block) => {
            block.body.split_last().is_some_and(|(last, preceding)| {
                matches!(last, Statement::ReturnStatement(_))
                    && returning_branch(last, name, source)
                    && preceding.iter().all(|statement| {
                        matches!(statement, Statement::ExpressionStatement(statement)
                        if checkpoint(&statement.expression, name, source))
                    })
            })
        }
        _ => false,
    }
}

fn checkpoint(expression: &Expression<'_>, name: &str, source: Option<&str>) -> bool {
    let expression = expression.get_inner_expression();
    let expression = match expression {
        Expression::AwaitExpression(value) => value.argument.get_inner_expression(),
        value => value,
    };
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    matches!(&call.callee, Expression::Identifier(identifier)
        if identifier.name != name && Some(identifier.name.as_str()) != source
            && identifier.name != "eval")
        && super::expressions::safe_arguments(call, name, source)
}

pub(super) fn read_only(expression: &Expression<'_>, name: &str, source: Option<&str>) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => {
            identifier.name != name && Some(identifier.name.as_str()) != source
        }
        Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::TemplateLiteral(template) => template
            .expressions
            .iter()
            .all(|value| read_only(value, name, source)),
        Expression::StaticMemberExpression(member) => read_only(&member.object, name, source),
        Expression::ComputedMemberExpression(member) => {
            read_only(&member.object, name, source) && read_only(&member.expression, name, source)
        }
        Expression::LogicalExpression(value) => {
            read_only(&value.left, name, source) && read_only(&value.right, name, source)
        }
        Expression::BinaryExpression(value) => {
            read_only(&value.left, name, source) && read_only(&value.right, name, source)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                read_only(&member.object, name, source)
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                read_only(&member.object, name, source)
                    && read_only(&member.expression, name, source)
            }
            _ => false,
        },
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
                if !property.computed && !property.method && property.kind == PropertyKind::Init
                    && read_only(&property.value, name, source))
        }),
        _ => false,
    }
}
