use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKind};

type Paths = Vec<Vec<String>>;

pub(super) fn catalog(expression: &Expression<'_>) -> Option<(bool, Paths, Paths)> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => {
            let paths = object_paths(object)?;
            let rows = object
                .properties
                .iter()
                .filter_map(ObjectPropertyKind::as_property)
                .map(|property| {
                    let Expression::ObjectExpression(row) =
                        crate::codebase::ts_source::unwrap_ts_wrappers(&property.value)
                    else {
                        return None;
                    };
                    object_paths(row)
                });
            Some((true, paths, intersect(rows)?))
        }
        Expression::ArrayExpression(array) => {
            let fields = intersect(array.elements.iter().map(|element| {
                let Expression::ObjectExpression(row) =
                    crate::codebase::ts_source::unwrap_ts_wrappers(element.as_expression()?)
                else {
                    return None;
                };
                object_paths(row)
            }))?;
            fields
                .iter()
                .any(|path| path.len() > 1)
                .then(|| (false, fields.clone(), fields))
        }
        _ => None,
    }
}

fn intersect(rows: impl Iterator<Item = Option<Paths>>) -> Option<Paths> {
    let mut common: Option<Paths> = None;
    for row in rows {
        let row = row?;
        common = Some(common.map_or_else(
            || row.clone(),
            |mut paths| {
                paths.retain(|path| row.contains(path));
                paths
            },
        ));
    }
    common.filter(|paths| !paths.is_empty())
}

fn object_paths(object: &ObjectExpression<'_>) -> Option<Paths> {
    let mut paths = Vec::new();
    let mut names = crate::fx::FxHashSet::default();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed
            || property.method
            || property.shorthand
            || property.kind != PropertyKind::Init
        {
            return None;
        }
        let key = crate::codebase::ts_source::static_property_key_name(&property.key)?.into_owned();
        if key == "__proto__" || !names.insert(key.clone()) {
            return None;
        }
        match crate::codebase::ts_source::unwrap_ts_wrappers(&property.value) {
            Expression::ObjectExpression(inner) => {
                for mut path in object_paths(inner)? {
                    path.insert(0, key.clone());
                    paths.push(path);
                }
            }
            value if super::super::closed_identifiers::literal_string(value).is_some() => {
                paths.push(vec![key]);
            }
            _ => return None,
        }
    }
    (!paths.is_empty()).then_some(paths)
}
