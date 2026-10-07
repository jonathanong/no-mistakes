use super::*;
use crate::codebase::dependencies::extract::static_import_specifier;
pub(super) fn dynamic(value: &ImportExpression<'_>, facts: &mut TypeScriptModuleFacts) {
    if let Some(specifier) = static_import_specifier(&value.source) {
        facts.loads.push(ModuleLoad {
            kind: "dynamicImport".into(),
            specifier,
            span: value.span.into(),
        });
    } else {
        unsupported(facts, value.span, "Non-literal dynamic import specifier");
    }
}
pub(super) fn require(
    value: &CallExpression<'_>,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    let Expression::Identifier(callee) = &value.callee else {
        return;
    };
    if callee.name == "eval" {
        unsupported(
            facts,
            value.span,
            "Dynamic eval may change binding semantics",
        );
        return;
    }
    if callee.name != "require" {
        return;
    }
    let reference = semantic
        .scoping()
        .get_reference(callee.reference_id.get().expect("semantic call reference"));
    if reference.symbol_id().is_some() {
        unsupported(facts, value.span, "Shadowed require is not a module loader");
        return;
    }
    let specifier = value
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(static_import_specifier);
    if let Some(specifier) = specifier {
        facts.loads.push(ModuleLoad {
            kind: "require".into(),
            specifier,
            span: value.span.into(),
        });
    } else {
        unsupported(
            facts,
            value.span,
            "Non-literal or unsupported require specifier",
        );
    }
}

pub(super) fn indirect(
    value: &IdentifierReference<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    if value.name != "require" {
        return;
    }
    let reference = semantic.scoping().get_reference(
        value
            .reference_id
            .get()
            .expect("semantic identifier reference"),
    );
    if !reference.is_value()
        || reference.flags().is_value_as_type()
        || reference.symbol_id().is_some()
    {
        return;
    }
    if let AstKind::CallExpression(call) = semantic.nodes().parent_kind(node_id) {
        if matches!(&call.callee, Expression::Identifier(callee) if callee.span == value.span) {
            return;
        }
    }
    unsupported(
        facts,
        value.span,
        "Indirect require reference is unsupported",
    );
}
