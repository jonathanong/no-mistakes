use super::ExportIndex;
use crate::ast::property_key_name;
use crate::queue::extract_model::RateLimitExport;
use oxc_ast::ast::{Class, Expression, ObjectExpression, ObjectPropertyKind};

pub(super) fn lookup_local(index: &ExportIndex, local: &str) -> RateLimitExport {
    if let Some(kind) = index.locals.get(local) {
        return kind.clone();
    }
    if let Some((source, imported)) = index.named_imports.get(local) {
        return RateLimitExport::FromNamed {
            source: source.clone(),
            imported: imported.clone(),
        };
    }
    index
        .ns_imports
        .get(local)
        .map_or(RateLimitExport::Opaque, |source| {
            RateLimitExport::FromNamespace {
                source: source.clone(),
            }
        })
}

pub(super) fn classify_class(class: &Class<'_>, index: &ExportIndex) -> RateLimitExport {
    let Some(heritage) = &class.heritage else {
        return RateLimitExport::NonQueue;
    };
    let Expression::Identifier(id) = heritage.expression.get_inner_expression() else {
        return RateLimitExport::Opaque;
    };
    let name = id.name.as_str();
    let non_queue = index.locals.get(name).map_or_else(
        || {
            !index.named_imports.contains_key(name)
                && !index.ns_imports.contains_key(name)
                && name != "RateLimitError"
        },
        |local| *local == RateLimitExport::NonQueue,
    );
    if non_queue {
        RateLimitExport::NonQueue
    } else {
        RateLimitExport::Opaque
    }
}

pub(super) fn classify_expr(expr: &Expression<'_>, index: &ExportIndex) -> RateLimitExport {
    match expr.get_inner_expression() {
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
            RateLimitExport::NonQueue
        }
        Expression::ClassExpression(class) => classify_class(class, index),
        Expression::Identifier(id) => lookup_local(index, id.name.as_str()),
        Expression::StaticMemberExpression(member) if member.property.name == "RateLimitError" => {
            classify_member_rate_limit(&member.object, index)
        }
        Expression::ObjectExpression(obj) => classify_object_export(obj, index),
        _ => RateLimitExport::Opaque,
    }
}

fn classify_member_rate_limit(object: &Expression<'_>, index: &ExportIndex) -> RateLimitExport {
    match object.get_inner_expression() {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            if let Some(source) = index.ns_imports.get(name) {
                return RateLimitExport::FromMemberRateLimit {
                    source: source.clone(),
                    imported: None,
                };
            }
            index
                .named_imports
                .get(name)
                .map_or(RateLimitExport::Opaque, |(source, imported)| {
                    RateLimitExport::FromMemberRateLimit {
                        source: source.clone(),
                        imported: Some(imported.clone()),
                    }
                })
        }
        Expression::StaticMemberExpression(inner)
            if matches!(inner.property.name.as_str(), "Worker" | "TestWorker") =>
        {
            let Expression::Identifier(ns) = inner.object.get_inner_expression() else {
                return RateLimitExport::Opaque;
            };
            index
                .ns_imports
                .get(ns.name.as_str())
                .map_or(RateLimitExport::Opaque, |source| {
                    RateLimitExport::FromMemberRateLimit {
                        source: source.clone(),
                        imported: Some(inner.property.name.to_string()),
                    }
                })
        }
        _ => RateLimitExport::Opaque,
    }
}

fn classify_object_export(obj: &ObjectExpression<'_>, index: &ExportIndex) -> RateLimitExport {
    let mut state = RateLimitExport::NonQueue;
    for prop in &obj.properties {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => {
                let Some(key) = property_key_name(&prop.key).filter(|_| !prop.computed) else {
                    state = RateLimitExport::Opaque;
                    continue;
                };
                if key == "RateLimitError" {
                    state = if classify_expr(&prop.value, index) == RateLimitExport::NonQueue {
                        RateLimitExport::NonQueue
                    } else {
                        RateLimitExport::Opaque
                    };
                }
            }
            ObjectPropertyKind::SpreadProperty(_) => state = RateLimitExport::Opaque,
        }
    }
    state
}
