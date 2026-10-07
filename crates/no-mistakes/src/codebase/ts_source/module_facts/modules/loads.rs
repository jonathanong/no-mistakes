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
    let Expression::Identifier(callee) = value.callee.get_inner_expression() else {
        return;
    };
    let reference = semantic
        .scoping()
        .get_reference(callee.reference_id.get().expect("semantic call reference"));
    if callee.name == "eval" && !value.optional && !has_runtime_binding(reference, semantic) {
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
    if has_runtime_binding(reference, semantic) {
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
        || has_runtime_binding(reference, semantic)
    {
        return;
    }
    let parent = semantic.nodes().ancestor_kinds(node_id).find(|kind| {
        !matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    });
    if let Some(AstKind::CallExpression(call)) = parent {
        if matches!(call.callee.get_inner_expression(), Expression::Identifier(callee) if callee.span == value.span)
        {
            return;
        }
    }
    unsupported(
        facts,
        value.span,
        "Indirect require reference is unsupported",
    );
}

fn has_runtime_binding(reference: &oxc_semantic::Reference, semantic: &Semantic<'_>) -> bool {
    reference
        .symbol_id()
        .is_some_and(|id| !semantic.scoping().symbol_flags(id).is_ambient())
}
