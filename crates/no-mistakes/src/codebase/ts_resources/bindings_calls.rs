use super::bindings::{fs_promise_method, import_binding, require_module, Binding};
use super::ResourceCallKind;
use oxc_ast::ast::Expression;
use std::collections::HashMap;

pub(super) fn nested_fs_promise_callee<'a>(
    callee: &'a Expression<'a>,
    bindings: &HashMap<String, Binding>,
) -> Option<(&'a str, ResourceCallKind)> {
    let Expression::StaticMemberExpression(method) = callee else {
        return None;
    };
    let Expression::StaticMemberExpression(promises) = &method.object else {
        return None;
    };
    let Expression::Identifier(namespace) = &promises.object else {
        return None;
    };
    if !matches!(
        bindings.get(namespace.name.as_str()),
        Some(Binding::FsNamespace)
    ) || promises.property.name != "promises"
    {
        return None;
    }
    Some((
        namespace.name.as_str(),
        fs_promise_method(method.property.name.as_str())?,
    ))
}

pub(super) fn inline_require_callee<'a>(
    callee: &'a Expression<'a>,
) -> Option<(&'a str, ResourceCallKind)> {
    let Expression::StaticMemberExpression(member) = callee else {
        return None;
    };
    if let Some(module) = require_module(&member.object) {
        return match import_binding(module, member.property.name.as_str()) {
            Some(Binding::FsMethod(kind) | Binding::GlobMethod(kind)) => Some(("require", kind)),
            _ => None,
        };
    }
    let Expression::StaticMemberExpression(promises) = &member.object else {
        return None;
    };
    let module = require_module(&promises.object)?;
    if !matches!(module, "fs" | "node:fs") || promises.property.name != "promises" {
        return None;
    }
    Some(("require", fs_promise_method(member.property.name.as_str())?))
}

pub(super) fn inline_require_file_url_to_path(callee: &Expression<'_>) -> bool {
    matches!(callee, Expression::StaticMemberExpression(member)
        if member.property.name == "fileURLToPath"
            && matches!(require_module(&member.object), Some("url" | "node:url")))
}
