use super::*;
impl ImportCollector {
    pub(super) fn object_value(&self, object: &ObjectExpression<'_>) -> ValueExpression {
        let mut fields = fx_map();
        let mut null_prototype = false;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return ValueExpression::Unknown;
            };
            if property.computed || property.method || property.kind != PropertyKind::Init {
                return ValueExpression::Unknown;
            }
            let Some(key) = write_targets::property_key(&property.key) else {
                return ValueExpression::Unknown;
            };
            if key == "__proto__" && !property.shorthand {
                if matches!(
                    crate::codebase::ts_source::unwrap_ts_wrappers(&property.value),
                    Expression::NullLiteral(_)
                ) {
                    null_prototype = true;
                    continue;
                }
                return ValueExpression::Unknown;
            }
            fields.insert(key, self.value_expression(&property.value));
        }
        ValueExpression::Object(std::sync::Arc::new(fields), null_prototype)
    }
}
