use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, ObjectPropertyKind};

pub(super) fn receiver<'a, 'b>(
    call: &'b CallExpression<'a>,
    properties: &[String],
) -> Option<Option<&'b Expression<'a>>> {
    if properties.is_empty() {
        return None;
    }
    let mut selected = None;
    for argument in call.arguments.iter().skip(1) {
        let Some(expression) = argument.as_expression() else {
            return Some(None);
        };
        let Expression::ObjectExpression(object) = unwrap_ts_wrappers(expression) else {
            if !matches!(
                unwrap_ts_wrappers(expression),
                Expression::ArrayExpression(_)
            ) {
                return Some(None);
            }
            continue;
        };
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return Some(None);
            };
            if property.computed {
                return Some(None);
            }
            let Some(name) = property.key.static_name() else {
                return Some(None);
            };
            if !properties
                .iter()
                .any(|candidate| candidate == name.as_ref())
            {
                continue;
            }
            if selected.is_some() || property.method {
                return Some(None);
            }
            let value = unwrap_ts_wrappers(&property.value);
            selected = Some(Some(match value {
                Expression::Identifier(_) => value,
                Expression::StaticMemberExpression(member) if member.property.name == "query" => {
                    unwrap_ts_wrappers(&member.object)
                }
                _ => return Some(None),
            }));
        }
    }
    selected
}
