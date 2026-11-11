use super::{PathOrigin, Projector};
use crate::fx::FxHashMap;
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn object_shape(
        &mut self,
        expression: &Expression<'a>,
        depth: usize,
    ) -> (FxHashMap<String, PathOrigin>, bool) {
        let mut paths = FxHashMap::default();
        let mut unknown = false;
        if depth >= 16 || !self.take() {
            return (paths, true);
        }
        let expression = match expression.get_inner_expression() {
            Expression::Identifier(id) => {
                let Some(symbol) = self.binding_symbol(id) else {
                    return (paths, true);
                };
                if !self.safe_uses(symbol, depth + 1) {
                    return (paths, true);
                }
                let Some(init) = self.const_init(id) else {
                    return (paths, true);
                };
                return self.object_shape(init, depth + 1);
            }
            other => other,
        };
        let Expression::ObjectExpression(object) = expression else {
            return (paths, true);
        };
        if object.properties.len() > 32 {
            return (paths, true);
        }
        for property in &object.properties {
            match property {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    let (spread_paths, opaque) = self.object_shape(&spread.argument, depth + 1);
                    // An unknown spread may replace any earlier property.
                    if opaque {
                        paths.retain(|key, _| self.spread_excludes(&spread.argument, key));
                        unknown = true;
                    }
                    paths.extend(spread_paths);
                }
                ObjectPropertyKind::ObjectProperty(property) => {
                    if property.method || property.kind != PropertyKind::Init {
                        return (FxHashMap::default(), true);
                    }
                    let key = (!property.computed)
                        .then(|| crate::ast::property_key_name(&property.key))
                        .flatten();
                    let Some(key) = key else {
                        paths.clear();
                        unknown = true;
                        continue;
                    };
                    if key == "__proto__" || key.contains('.') {
                        return (FxHashMap::default(), true);
                    }
                    let mut origin = self.origin(&property.value, depth + 1);
                    if object.span.start < self.call_offset
                        && origin
                            .atoms()
                            .iter()
                            .any(|atom| atom.env.is_some() && atom.env_capture.is_none())
                    {
                        origin = PathOrigin::unknown();
                    }
                    paths.insert(key.into_owned(), origin);
                }
            }
        }
        (paths, unknown)
    }
}
