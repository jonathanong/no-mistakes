use super::super::{ArgumentValue, PathProvenance, Projector};
use oxc_ast::ast::*;

const LOADERS: [&str; 8] = [
    "importActual",
    "importMock",
    "requireActual",
    "requireMock",
    "mock",
    "doMock",
    "unmock",
    "doUnmock",
];

pub(super) fn from_call(
    projector: &mut Projector<'_, '_>,
    call: &CallExpression<'_>,
    out: &mut PathProvenance,
) {
    let loads = match call.callee.get_inner_expression() {
        Expression::Identifier(callee) => callee.name == "require",
        Expression::StaticMemberExpression(callee) => {
            callee.property.name == "require" || LOADERS.contains(&callee.property.name.as_str())
        }
        _ => false,
    };
    if loads {
        let source = call.arguments.first().and_then(Argument::as_expression);
        if !matches!(
            source.map(Expression::get_inner_expression),
            Some(Expression::ImportExpression(_))
        ) {
            from_source(projector, source, out);
        }
    }
}

pub(super) fn from_source(
    projector: &mut Projector<'_, '_>,
    source: Option<&Expression<'_>>,
    out: &mut PathProvenance,
) -> bool {
    match source.map(|source| projector.value(source, 0)) {
        Some(ArgumentValue::Scalar(serde_json::Value::String(text))) => {
            super::process_refs::poison_process_loads(&text, out);
            out.unsafe_modules.insert(text);
            true
        }
        _ => {
            out.modules_unknown = true;
            out.poison_env();
            false
        }
    }
}

pub(super) fn from_facts(
    facts: &super::super::super::TypeScriptModuleFacts,
    out: &mut PathProvenance,
) {
    for import in &facts.imports {
        let unsafe_binding = import.bindings.iter().any(|binding| {
            binding.kind != "named"
                || binding.imported == "default"
                || out.value_refs.contains(&binding.local)
        });
        if unsafe_binding {
            out.unsafe_modules.insert(import.specifier.clone());
        }
    }
    for export in &facts.exports {
        if let Some(specifier) = &export.specifier {
            let named = !export.type_only
                && [&export.local, &export.exported]
                    .iter()
                    .all(|name| !matches!(name.as_str(), "" | "*" | "default"));
            if !named {
                out.unsafe_modules.insert(specifier.clone());
            }
        }
    }
}
