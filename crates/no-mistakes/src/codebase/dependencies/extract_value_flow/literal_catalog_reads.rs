use super::*;
use oxc_ast::ast::{Expression, IdentifierReference};

impl ImportCollector {
    pub(crate) fn note_literal_catalog_expression(&mut self, expression: &Expression<'_>) {
        if !self.collect_value_flow || self.namespace.type_depth > 0 {
            return;
        }
        let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
        if let Expression::ArrowFunctionExpression(arrow) = expression {
            if let Some(callback) = callbacks::callback(arrow) {
                self.value_flow
                    .catalogs
                    .callbacks
                    .insert(CallableId(arrow.span.start), callback);
            }
        }
        let mut kind = CatalogReadKind::Value;
        let value = if let Expression::CallExpression(call) = expression {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if matches!(member.property.name.as_str(), "filter" | "map" | "join") {
                    if let Expression::CallExpression(base) =
                        crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
                    {
                        self.value_flow
                            .catalogs
                            .consumers
                            .insert(base.span.start, (call.span.start, call.span.end));
                    }
                }
            }
            kind = match &call.callee {
                Expression::Identifier(_) if !call.optional => CatalogReadKind::Callee,
                Expression::StaticMemberExpression(member)
                    if !call.optional && !member.optional
                        && matches!(member.property.name.as_str(), "filter" | "map")
                        && call.arguments.len() == 1
                        && call.arguments[0].as_expression().is_some_and(|argument| {
                            matches!(crate::codebase::ts_source::unwrap_ts_wrappers(argument),
                                Expression::ArrowFunctionExpression(arrow) if callbacks::callback(arrow).is_some())
                        }) => CatalogReadKind::Collection(call.span.start, call.span.end),
                _ => CatalogReadKind::Invalid,
            };
            &call.callee
        } else {
            expression
        };
        let Some((identifier, mut path)) = root(value) else {
            return;
        };
        if self.metadata_rows.writing {
            kind = CatalogReadKind::Invalid;
        }
        if matches!(kind, CatalogReadKind::Collection(_, _)) {
            path.pop();
        }
        let Some(scope) = self.catalog_binding_scope(identifier.name.as_str()) else {
            return;
        };
        self.value_flow
            .catalogs
            .reads
            .entry(identifier.span.start)
            .or_insert_with(|| CatalogRead {
                name: identifier.name.to_string(),
                scope,
                path,
                kind,
            });
    }
}

fn root<'a>(expression: &'a Expression<'a>) -> Option<(&'a IdentifierReference<'a>, Vec<String>)> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => Some((identifier.as_ref(), Vec::new())),
        Expression::StaticMemberExpression(member) if !member.optional => {
            let (identifier, mut path) = root(&member.object)?;
            path.push(member.property.name.to_string());
            Some((identifier, path))
        }
        Expression::ComputedMemberExpression(member) if !member.optional => {
            let (identifier, mut path) = root(&member.object)?;
            path.push(super::super::closed_identifiers::literal_string(
                &member.expression,
            )?);
            Some((identifier, path))
        }
        _ => None,
    }
}
