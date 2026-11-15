use super::{PathOrigin, Projector};
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(super) fn object_member_origin(
        &mut self,
        expression: &Expression<'a>,
        object: &ObjectExpression<'a>,
        key: &str,
        depth: usize,
    ) -> Option<PathOrigin> {
        if object
            .properties
            .iter()
            .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
        {
            return self.object_shape(expression, depth + 1).0.remove(key);
        }
        let mut value = None;
        for property in object
            .properties
            .iter()
            .filter_map(ObjectPropertyKind::as_property)
        {
            if property.computed || property.method || property.kind != PropertyKind::Init {
                return None;
            }
            if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                value = Some(&property.value);
            }
        }
        value.map(|value| self.origin(value, depth + 1))
    }
}
