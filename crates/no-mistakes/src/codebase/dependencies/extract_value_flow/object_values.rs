use super::*;
#[cfg(feature = "test-instrumentation")]
thread_local! {
    static ROOT_RHS_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static ROOT_RHS_MAPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
impl ImportCollector {
    pub(super) fn root_value_expression(&self, expression: &Expression<'_>) -> ValueExpression {
        #[cfg(feature = "test-instrumentation")]
        let previous = ROOT_RHS_ACTIVE.with(|active| active.replace(true));
        let value = self.value_expression(expression);
        #[cfg(feature = "test-instrumentation")]
        ROOT_RHS_ACTIVE.with(|active| active.set(previous));
        value
    }
    pub(super) fn object_value(&self, object: &ObjectExpression<'_>) -> ValueExpression {
        #[cfg(feature = "test-instrumentation")]
        if ROOT_RHS_ACTIVE.with(std::cell::Cell::get) {
            ROOT_RHS_MAPS.with(|count| count.set(count.get() + 1));
        }
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
