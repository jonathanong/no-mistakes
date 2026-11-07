use super::{PathOrigin, Projector, MAX_ARGUMENTS};
use oxc_ast::ast::*;

pub(super) fn collect<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
) -> Vec<(usize, String, PathOrigin)> {
    let mut members = Vec::new();
    for (index, argument) in call.arguments.iter().take(MAX_ARGUMENTS).enumerate() {
        let Some(Expression::ObjectExpression(object)) = argument
            .as_expression()
            .map(Expression::get_inner_expression)
        else {
            continue;
        };
        if object.properties.len() > 32 {
            continue;
        }
        let mut paths = crate::fx::FxHashMap::default();
        let mut valid = true;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                valid = false;
                break;
            };
            if property.computed || property.method || property.kind != PropertyKind::Init {
                valid = false;
                break;
            }
            if let Some(key) = crate::ast::property_key_name(&property.key) {
                paths.insert(key.to_string(), projector.path_of(&property.value));
            }
        }
        if valid {
            members.extend(paths.into_iter().map(|(key, origin)| (index, key, origin)));
        }
    }
    members.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    members
}
