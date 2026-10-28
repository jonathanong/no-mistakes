use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKind};

pub(super) fn literal_map(init: &Expression<'_>) -> Option<FxHashMap<String, (String, u32)>> {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(init) else {
        return None;
    };
    let mut rows = FxHashMap::default();
    for item in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = item else {
            return None;
        };
        if property.computed
            || property.method
            || property.shorthand
            || property.kind != PropertyKind::Init
        {
            return None;
        }
        let key = static_property_key_name(&property.key)?.to_string();
        if key == "__proto__" {
            return None;
        }
        let Expression::ObjectExpression(record) = unwrap_ts_wrappers(&property.value) else {
            return None;
        };
        let mut sql = None;
        for item in &record.properties {
            let ObjectPropertyKind::ObjectProperty(field) = item else {
                return None;
            };
            if field.computed || field.method || field.kind != PropertyKind::Init {
                return None;
            }
            let name = static_property_key_name(&field.key)?;
            if name == "__proto__" {
                return None;
            }
            if name == "sql" {
                if field.shorthand || sql.is_some() {
                    return None;
                }
                let Expression::StringLiteral(value) = unwrap_ts_wrappers(&field.value) else {
                    return None;
                };
                sql = Some((crate::ast::js_literal(value).into_owned(), value.span.start));
            }
        }
        if rows.insert(key, sql?).is_some() {
            return None;
        }
    }
    (!rows.is_empty()).then_some(rows)
}
