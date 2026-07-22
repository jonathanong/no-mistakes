use super::ResourceCallKind;
use oxc_ast::ast::{Argument, BindingPattern, Expression};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    FsNamespace,
    FsPromisesNamespace,
    FsMethod(ResourceCallKind),
    GlobNamespace,
    GlobMethod(ResourceCallKind),
    UrlNamespace,
    UrlConstructor,
    FileUrlToPath,
}

pub(super) fn import_binding(module: &str, imported: &str) -> Option<Binding> {
    match module {
        "fs" | "node:fs" => match imported {
            "default" | "*" => Some(Binding::FsNamespace),
            "promises" => Some(Binding::FsPromisesNamespace),
            name => fs_method(name).map(Binding::FsMethod),
        },
        "fs/promises" | "node:fs/promises" => match imported {
            "default" | "*" => Some(Binding::FsPromisesNamespace),
            name => fs_promise_method(name).map(Binding::FsMethod),
        },
        "glob" | "fast-glob" | "tinyglobby" => match imported {
            "default" => Some(Binding::GlobMethod(ResourceCallKind::Glob)),
            "*" => Some(Binding::GlobNamespace),
            name => glob_method(name).map(Binding::GlobMethod),
        },
        "url" | "node:url" => match imported {
            "default" | "*" => Some(Binding::UrlNamespace),
            "URL" => Some(Binding::UrlConstructor),
            "fileURLToPath" => Some(Binding::FileUrlToPath),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn fs_method(name: &str) -> Option<ResourceCallKind> {
    match name {
        "readFile" => Some(ResourceCallKind::ReadFile),
        "readFileSync" => Some(ResourceCallKind::ReadFileSync),
        "readdir" => Some(ResourceCallKind::ReadDirectory),
        "readdirSync" => Some(ResourceCallKind::ReadDirectorySync),
        _ => None,
    }
}

pub(super) fn fs_promise_method(name: &str) -> Option<ResourceCallKind> {
    match name {
        "readFile" => Some(ResourceCallKind::ReadFile),
        "readdir" => Some(ResourceCallKind::ReadDirectory),
        _ => None,
    }
}

pub(super) fn glob_method(name: &str) -> Option<ResourceCallKind> {
    match name {
        "glob" => Some(ResourceCallKind::Glob),
        "sync" | "globSync" => Some(ResourceCallKind::GlobSync),
        _ => None,
    }
}

pub(super) fn binding_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => vec![id.name.to_string()],
        BindingPattern::ObjectPattern(object) => object
            .properties
            .iter()
            .flat_map(|property| binding_names(&property.value))
            .collect(),
        BindingPattern::ArrayPattern(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(binding_names)
            .collect(),
        BindingPattern::AssignmentPattern(assignment) => binding_names(&assignment.left),
    }
}

pub(super) fn require_module<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    (callee.name == "require")
        .then(|| match call.arguments.first() {
            Some(Argument::StringLiteral(value)) => Some(value.value.as_str()),
            _ => None,
        })
        .flatten()
}

pub(super) fn require_module_or_promises<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    if let Some(module) = require_module(expr) {
        return Some(module);
    }
    let Expression::StaticMemberExpression(member) = expr else {
        return None;
    };
    (member.property.name == "promises")
        .then(|| require_module(&member.object))
        .flatten()
        .and_then(|module| matches!(module, "fs" | "node:fs").then_some("node:fs/promises"))
}

pub(super) fn require_member_binding(expr: &Expression<'_>) -> Option<Binding> {
    let Expression::StaticMemberExpression(member) = expr else {
        return None;
    };
    import_binding(
        require_module_or_promises(&member.object)?,
        member.property.name.as_str(),
    )
}

pub(super) fn register_require_binding(
    pattern: &BindingPattern<'_>,
    module: &str,
    bindings: &mut HashMap<String, Binding>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            if let Some(binding) = import_binding(module, "default") {
                bindings.insert(id.name.to_string(), binding);
            }
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                let Some(key) = property.key.static_name() else {
                    continue;
                };
                match &property.value {
                    BindingPattern::BindingIdentifier(id) => {
                        if let Some(binding) = import_binding(module, key.as_ref()) {
                            bindings.insert(id.name.to_string(), binding);
                        }
                    }
                    BindingPattern::ObjectPattern(inner)
                        if key.as_ref() == "promises" && matches!(module, "fs" | "node:fs") =>
                    {
                        for nested in &inner.properties {
                            let (Some(key), BindingPattern::BindingIdentifier(id)) =
                                (nested.key.static_name(), &nested.value)
                            else {
                                continue;
                            };
                            if let Some(binding) = import_binding("node:fs/promises", key.as_ref())
                            {
                                bindings.insert(id.name.to_string(), binding);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
