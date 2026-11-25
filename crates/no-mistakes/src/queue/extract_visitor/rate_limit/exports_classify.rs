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
    if let Some(source) = index.ns_imports.get(local) {
        return RateLimitExport::FromNamespace {
            source: source.clone(),
        };
    }
    RateLimitExport::Opaque
}

pub(super) fn classify_class(class: &Class<'_>, index: &ExportIndex) -> RateLimitExport {
    let Some(heritage) = &class.heritage else {
        return RateLimitExport::NonQueue;
    };
    let Expression::Identifier(id) = heritage.expression.get_inner_expression() else {
        return RateLimitExport::Opaque;
    };
    let name = id.name.as_str();
    if let Some(local) = index.locals.get(name) {
        return if *local == RateLimitExport::NonQueue {
            RateLimitExport::NonQueue
        } else {
            RateLimitExport::Opaque
        };
    }
    if index.named_imports.contains_key(name)
        || index.ns_imports.contains_key(name)
        || name == "RateLimitError"
    {
        RateLimitExport::Opaque
    } else {
        RateLimitExport::NonQueue
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
            if let Some((source, imported)) = index.named_imports.get(name) {
                return RateLimitExport::FromMemberRateLimit {
                    source: source.clone(),
                    imported: Some(imported.clone()),
                };
            }
            RateLimitExport::Opaque
        }
        Expression::StaticMemberExpression(inner)
            if matches!(inner.property.name.as_str(), "Worker" | "TestWorker") =>
        {
            if let Expression::Identifier(ns) = inner.object.get_inner_expression() {
                if let Some(source) = index.ns_imports.get(ns.name.as_str()) {
                    return RateLimitExport::FromMemberRateLimit {
                        source: source.clone(),
                        imported: Some(inner.property.name.to_string()),
                    };
                }
            }
            RateLimitExport::Opaque
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
