use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use std::sync::Arc;

pub(super) fn receiver_label(expr: &Expression<'_>) -> Option<Arc<str>> {
    let expr = unwrap_ts_wrappers(expr);
    if let Expression::Identifier(id) = expr {
        return Some(Arc::from(id.name.as_str()));
    }
    let member = expr.as_member_expression()?;
    let property = member.static_property_name()?.as_str()?;
    let label = match unwrap_ts_wrappers(member.object()) {
        Expression::Identifier(id) => format!("{}.{property}", id.name.as_str()),
        Expression::ThisExpression(_) => format!("this.{property}"),
        Expression::NewExpression(instance) => {
            let Expression::Identifier(class) = unwrap_ts_wrappers(&instance.callee) else {
                return Some(Arc::from(property));
            };
            format!("new {}().{property}", class.name.as_str())
        }
        _ => return Some(Arc::from(property)),
    };
    Some(Arc::from(label))
}
