use super::{Binding, ConstArrays};
use crate::codebase::rules::csp_allowed_origins::findings::DeferredPart;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, CallExpression, Expression, TemplateLiteral,
};
use std::path::PathBuf;

pub(in super::super) enum RegistryJoin {
    Resolved(String),
    Deferred(Vec<DeferredPart>),
}

enum Piece {
    Text(String),
    Import {
        module: PathBuf,
        export_name: String,
    },
}

impl ConstArrays {
    pub(in crate::codebase::rules::csp_allowed_origins::extract) fn eval_join(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<RegistryJoin> {
        let (object, separator) = static_join(call)?;
        match unwrap_ts_wrappers(object) {
            Expression::Identifier(identifier) => {
                self.join_name(identifier.name.as_str(), &separator)
            }
            Expression::ArrayExpression(array) => self.join_spread_array(array, &separator),
            _ => None,
        }
    }

    pub(in crate::codebase::rules::csp_allowed_origins::extract) fn defer_template(
        &self,
        template: &TemplateLiteral<'_>,
    ) -> Option<Vec<DeferredPart>> {
        if template.expressions.is_empty() {
            return None;
        }
        let mut parts = Vec::new();
        let mut saw_import = false;
        for (idx, quasi) in template.quasis.iter().enumerate() {
            quasi.value.cooked.as_ref()?;
            parts.push(DeferredPart::Text(
                crate::ast::template_quasi_text(quasi).into_owned(),
            ));
            let Some(expr) = template.expressions.get(idx) else {
                break;
            };
            match self.interpolation(expr)? {
                RegistryJoin::Resolved(text) => parts.push(DeferredPart::Text(text)),
                RegistryJoin::Deferred(extra) => {
                    saw_import = true;
                    parts.extend(extra);
                }
            }
        }
        saw_import.then_some(parts)
    }

    fn interpolation(&self, expr: &Expression<'_>) -> Option<RegistryJoin> {
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) {
            if let Some(joined) = self.eval_join(call) {
                return Some(joined);
            }
        }
        super::super::csp_strings::eval_csp_text(expr, self)
            .map(|text| RegistryJoin::Resolved(text.into_owned()))
    }

    fn join_name(&self, name: &str, separator: &str) -> Option<RegistryJoin> {
        match self.bindings.get(name)? {
            Binding::Local(items) => Some(RegistryJoin::Resolved(items.join(separator))),
            Binding::Imported {
                module,
                export_name,
            } => Some(RegistryJoin::Deferred(vec![DeferredPart::Origins {
                module: module.clone(),
                export_name: export_name.clone(),
                separator: separator.to_string(),
            }])),
        }
    }

    fn join_spread_array(
        &self,
        array: &ArrayExpression<'_>,
        separator: &str,
    ) -> Option<RegistryJoin> {
        let mut saw_registry = false;
        let mut pieces = Vec::new();
        for element in &array.elements {
            match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    let Expression::Identifier(identifier) = unwrap_ts_wrappers(&spread.argument)
                    else {
                        return None;
                    };
                    match self.bindings.get(identifier.name.as_str())? {
                        Binding::Local(items) => {
                            saw_registry = true;
                            pieces.extend(items.iter().cloned().map(Piece::Text));
                        }
                        Binding::Imported {
                            module,
                            export_name,
                        } => {
                            saw_registry = true;
                            pieces.push(Piece::Import {
                                module: module.clone(),
                                export_name: export_name.clone(),
                            });
                        }
                    }
                }
                other => {
                    let text =
                        super::super::csp_strings::eval_static_string(other.as_expression()?)?
                            .into_owned();
                    pieces.push(Piece::Text(text));
                }
            }
        }
        saw_registry.then(|| flatten(pieces, separator))
    }
}

fn static_join<'a>(call: &'a CallExpression<'a>) -> Option<(&'a Expression<'a>, String)> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
        return None;
    }
    let separator = match call.arguments.first() {
        None => ",".to_string(),
        Some(argument) => {
            super::super::csp_strings::eval_static_string(argument.as_expression()?)?.into_owned()
        }
    };
    Some((&member.object, separator))
}

fn flatten(pieces: Vec<Piece>, separator: &str) -> RegistryJoin {
    let mut saw_import = false;
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut started = false;
    for piece in pieces {
        if started {
            text.push_str(separator);
        }
        started = true;
        match piece {
            Piece::Text(item) => text.push_str(&item),
            Piece::Import {
                module,
                export_name,
            } => {
                if !text.is_empty() {
                    parts.push(DeferredPart::Text(std::mem::take(&mut text)));
                }
                saw_import = true;
                parts.push(DeferredPart::Origins {
                    module,
                    export_name,
                    separator: separator.to_string(),
                });
            }
        }
    }
    if saw_import {
        if !text.is_empty() {
            parts.push(DeferredPart::Text(text));
        }
        RegistryJoin::Deferred(parts)
    } else {
        RegistryJoin::Resolved(text)
    }
}
