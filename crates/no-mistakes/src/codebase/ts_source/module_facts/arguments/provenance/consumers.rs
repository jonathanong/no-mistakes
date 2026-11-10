use super::super::{line_in, ArgumentValue, PathProvenance, Projector, RouteKind, RouteSite};
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
            from_source(projector, source, out, call.span.start, load_kind(call));
        }
    }
}

fn load_kind(call: &CallExpression<'_>) -> RouteKind {
    match call.callee.get_inner_expression() {
        Expression::StaticMemberExpression(callee)
            if matches!(
                callee.property.name.as_str(),
                "mock" | "doMock" | "unmock" | "doUnmock"
            ) =>
        {
            RouteKind::ViMock
        }
        _ => RouteKind::Require,
    }
}

pub(super) fn from_source(
    projector: &mut Projector<'_, '_>,
    source: Option<&Expression<'_>>,
    out: &mut PathProvenance,
    start: u32,
    kind: RouteKind,
) -> bool {
    match source.map(|source| projector.value(source, 0)) {
        Some(ArgumentValue::Scalar(serde_json::Value::String(text))) => {
            super::process_refs::poison_process_loads(&text, out);
            out.unsafe_modules.insert(text.clone());
            out.route_sites.push(RouteSite::at(
                projector.semantic.source_text(),
                start,
                kind,
                text,
            ));
            true
        }
        _ => {
            out.modules_unknown = true;
            out.poison_env();
            let line = line_in(projector.semantic.source_text(), start);
            out.unknown_load = Some(out.unknown_load.map_or(line, |prev| prev.min(line)));
            out.route_sites.push(RouteSite::at(
                projector.semantic.source_text(),
                start,
                kind,
                String::new(),
            ));
            false
        }
    }
}

pub(super) fn from_facts(
    facts: &super::super::super::TypeScriptModuleFacts,
    source: &str,
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
        let line = line_in(source, import.span.start);
        out.import_lines
            .entry(import.specifier.clone())
            .and_modify(|prior| *prior = (*prior).min(line))
            .or_insert(line);
        for binding in &import.bindings {
            let kind = if binding.kind == "namespace" {
                Some(RouteKind::Namespace)
            } else if binding.kind == "default" || binding.imported == "default" {
                Some(RouteKind::Default)
            } else {
                None
            };
            if let Some(kind) = kind {
                out.route_sites.push(RouteSite::at(
                    source,
                    binding.span.start,
                    kind,
                    import.specifier.clone(),
                ));
            }
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
                let kind = if export.local == "*" || export.exported == "*" {
                    RouteKind::Namespace
                } else {
                    RouteKind::Default
                };
                out.route_sites.push(RouteSite::at(
                    source,
                    export.span.start,
                    kind,
                    specifier.clone(),
                ));
            }
        }
    }
}
