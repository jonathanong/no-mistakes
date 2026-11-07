use super::CatalogCallback;
use oxc_ast::ast::{ArrowFunctionExpression, BindingPattern, Expression};

pub(super) fn callback(arrow: &ArrowFunctionExpression<'_>) -> Option<CatalogCallback> {
    if arrow.r#async || arrow.params.items.len() != 1 || arrow.params.rest.is_some() {
        return None;
    }
    let BindingPattern::BindingIdentifier(parameter) = &arrow.params.items[0].pattern else {
        return None;
    };
    let expression = arrow.body.as_expression()?;
    let mut reads = Vec::new();
    if !pure(expression, parameter.name.as_str(), &mut reads) {
        return None;
    }
    reads.sort_unstable();
    reads.dedup();
    Some(CatalogCallback {
        reads,
        string_output: string_output(expression),
    })
}

fn pure(expression: &Expression<'_>, parameter: &str, reads: &mut Vec<Vec<String>>) -> bool {
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    if expression.is_literal() {
        return true;
    }
    if let Some(path) = member_path(expression, parameter) {
        reads.push(path);
        return true;
    }
    match expression {
        Expression::TemplateLiteral(template) => template
            .expressions
            .iter()
            .all(|part| pure(part, parameter, reads)),
        Expression::ConditionalExpression(branch) => {
            pure(&branch.test, parameter, reads)
                && pure(&branch.consequent, parameter, reads)
                && pure(&branch.alternate, parameter, reads)
        }
        Expression::BinaryExpression(binary) => {
            pure(&binary.left, parameter, reads) && pure(&binary.right, parameter, reads)
        }
        Expression::LogicalExpression(logical) => {
            pure(&logical.left, parameter, reads) && pure(&logical.right, parameter, reads)
        }
        Expression::UnaryExpression(unary) => {
            unary.operator == oxc_syntax::operator::UnaryOperator::LogicalNot
                && pure(&unary.argument, parameter, reads)
        }
        _ => false,
    }
}

fn string_output(expression: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_) => true,
        Expression::ConditionalExpression(branch) => {
            string_output(&branch.consequent) && string_output(&branch.alternate)
        }
        Expression::BinaryExpression(binary) => {
            binary.operator == oxc_syntax::operator::BinaryOperator::Addition
                && string_output(&binary.left)
                && string_output(&binary.right)
        }
        _ => false,
    }
}

fn member_path(expression: &Expression<'_>, parameter: &str) -> Option<Vec<String>> {
    fn collect(expression: &Expression<'_>, parameter: &str, path: &mut Vec<String>) -> Option<()> {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) if identifier.name == parameter => Some(()),
            Expression::StaticMemberExpression(member) if !member.optional => {
                collect(&member.object, parameter, path)?;
                path.push(member.property.name.to_string());
                Some(())
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                collect(&member.object, parameter, path)?;
                path.push(super::super::closed_identifiers::literal_string(
                    &member.expression,
                )?);
                Some(())
            }
            _ => None,
        }
    }
    let mut path = Vec::new();
    collect(expression, parameter, &mut path)?;
    (!path.is_empty()).then_some(path)
}
