use super::super::PathProvenance;
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

pub(super) fn from_call(call: &CallExpression<'_>, out: &mut PathProvenance) {
    let loads = match call.callee.get_inner_expression() {
        Expression::Identifier(callee) => callee.name == "require",
        Expression::StaticMemberExpression(callee) => {
            LOADERS.contains(&callee.property.name.as_str())
        }
        _ => false,
    };
    if loads {
        let source = call.arguments.first().and_then(Argument::as_expression);
        from_source(source, out);
    }
}

pub(super) fn from_source(source: Option<&Expression<'_>>, out: &mut PathProvenance) {
    match source.map(Expression::get_inner_expression) {
        Some(Expression::StringLiteral(text)) => {
            out.unsafe_modules
                .insert(crate::ast::js_literal(text).into_owned());
        }
        _ => out.modules_unknown = true,
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
            out.unsafe_modules.insert(specifier.clone());
        }
    }
}
