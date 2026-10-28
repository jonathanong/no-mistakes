//! Exact Vitest iterator keys and local immutable SQL map selection.
use super::super::map::literal_map;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{self, BindingPattern, Expression, Statement};

pub(super) fn keys(
    call: &ast::CallExpression<'_>,
    imports: &FxHashSet<String>,
    shadowed: impl Fn(&str) -> bool,
) -> Option<Vec<String>> {
    if call.optional || call.arguments.len() != 2 {
        return None;
    }
    let Expression::CallExpression(each) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if each.optional || each.arguments.len() != 1 {
        return None;
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&each.callee) else {
        return None;
    };
    let Expression::Identifier(it) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    if member.optional
        || member.property.name != "each"
        || !imports.contains(it.name.as_str())
        || shadowed(it.name.as_str())
    {
        return None;
    }
    let Expression::ArrayExpression(array) =
        unwrap_ts_wrappers(each.arguments.first()?.as_expression()?)
    else {
        return None;
    };
    if array.elements.is_empty() || array.elements.len() > 16 {
        return None;
    }
    let mut keys = Vec::new();
    let mut unique = FxHashSet::default();
    for item in &array.elements {
        let Expression::StringLiteral(value) = unwrap_ts_wrappers(item.as_expression()?) else {
            return None;
        };
        let key = crate::ast::js_literal(value).into_owned();
        if !unique.insert(key.clone()) {
            return None;
        }
        keys.push(key);
    }
    Some(keys)
}

pub(super) struct Selection {
    pub map: String,
    pub alias: String,
    pub values: Vec<(String, u32)>,
    pub allowed: FxHashSet<u32>,
    pub declarations: FxHashSet<u32>,
}

pub(super) fn selection(
    body: &ast::FunctionBody<'_>,
    key: &str,
    keys: Vec<String>,
) -> Option<Selection> {
    let mut map = None;
    let mut alias = None;
    for statement in &body.statements {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if !declaration.kind.is_const() {
            continue;
        }
        for item in &declaration.declarations {
            let (BindingPattern::BindingIdentifier(id), Some(init)) = (&item.id, &item.init) else {
                continue;
            };
            if map.is_none() {
                if let Some(values) = literal_map(init) {
                    map = Some((id, values));
                }
            } else if alias.is_none() {
                if let Expression::ComputedMemberExpression(select) = unwrap_ts_wrappers(init) {
                    if let (Expression::Identifier(root), Expression::Identifier(field)) = (
                        unwrap_ts_wrappers(&select.object),
                        unwrap_ts_wrappers(&select.expression),
                    ) {
                        if !select.optional
                            && root.name == map.as_ref()?.0.name
                            && field.name == key
                        {
                            alias = Some((id, root.span.start, field.span.start));
                        }
                    }
                }
            }
        }
    }
    let (map_id, map_values): (_, FxHashMap<String, (String, u32)>) = map?;
    let (alias_id, map_ref, key_ref) = alias?;
    // The ordered walk records an alias only after its map declaration.
    let mut values = Vec::new();
    for key in keys {
        values.push(map_values.get(&key)?.clone());
    }
    let mut allowed = FxHashSet::default();
    allowed.insert(map_ref);
    allowed.insert(key_ref);
    Some(Selection {
        map: map_id.name.to_string(),
        alias: alias_id.name.to_string(),
        values,
        allowed,
        declarations: [map_id.span.start, alias_id.span.start]
            .into_iter()
            .collect(),
    })
}
