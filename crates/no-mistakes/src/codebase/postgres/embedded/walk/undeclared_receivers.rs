use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn receiver_path<'e>(expression: &'e Expression<'_>) -> Option<(&'e str, String)> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(name) => Some((name.name.as_str(), String::new())),
        Expression::ThisExpression(_) => Some(("this", String::new())),
        Expression::StaticMemberExpression(member) => {
            let (name, mut path) = receiver_path(&member.object)?;
            path.push('.');
            path.push_str(member.property.name.as_str());
            Some((name, path))
        }
        Expression::ComputedMemberExpression(member) => {
            let (name, mut path) = receiver_path(&member.object)?;
            match unwrap_ts_wrappers(&member.expression) {
                Expression::StringLiteral(key) => {
                    path.push('.');
                    path.push_str(crate::ast::js_literal(key).as_ref());
                }
                _ => path.push_str("[]"),
            }
            Some((name, path))
        }
        _ => None,
    }
}
