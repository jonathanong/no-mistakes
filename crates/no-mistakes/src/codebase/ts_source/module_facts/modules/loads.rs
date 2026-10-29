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
pub(super) fn require<'a>(
    value: &CallExpression<'a>,
    semantic: &Semantic<'_>,
    loaders: &mut super::super::node_loaders::Collector<'_, 'a>,
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
    let bound = has_runtime_binding(reference, semantic);
    if bound && loaders.created_require(callee) {
        created_call(value, callee.name == "require", facts);
        return;
    }
    if callee.name != "require" {
        return;
    }
    if bound {
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

/// Public facts stay as on a shadowed `require`; the importer rule gets the
/// literal load and the shadowed-require diagnostic is marked ignorable.
fn created_call(value: &CallExpression<'_>, shadowed: bool, facts: &mut TypeScriptModuleFacts) {
    if shadowed {
        unsupported(facts, value.span, "Shadowed require is not a module loader");
    }
    let specifier = value
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(static_import_specifier);
    let Some(specifier) = specifier else {
        let message = "Non-literal or unsupported require specifier";
        facts.importer_unsafe.push((value.span.start, message));
        return;
    };
    if shadowed {
        facts.importer_safe.push((value.span.start, value.span.end));
    }
    facts.created_requires.push(ModuleLoad {
        kind: "require".into(),
        specifier,
        span: value.span.into(),
    });
}

/// A reference to a created loader that is neither a direct call nor
/// `.resolve` can load anything, so the importer rule cannot follow it.
pub(super) fn created_escape<'a>(
    value: &IdentifierReference<'a>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    loaders: &mut super::super::node_loaders::Collector<'_, 'a>,
    facts: &mut TypeScriptModuleFacts,
) {
    let reference = semantic.scoping().get_reference(
        value
            .reference_id
            .get()
            .expect("semantic identifier reference"),
    );
    if !reference.is_value()
        || reference.flags().is_value_as_type()
        || !has_runtime_binding(reference, semantic)
        || !loaders.created_require(value)
    {
        return;
    }
    let direct = match unwrapped_parent(node_id, semantic) {
        Some(AstKind::CallExpression(call)) => {
            matches!(call.callee.get_inner_expression(), Expression::Identifier(callee) if callee.span == value.span)
        }
        Some(AstKind::StaticMemberExpression(member)) => member.property.name == "resolve",
        _ => false,
    };
    if !direct {
        let message = "Created require loader escapes direct calls";
        facts.importer_unsafe.push((value.span.start, message));
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
    let parent = unwrapped_parent(node_id, semantic);
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

pub(in super::super) fn has_runtime_binding(
    reference: &oxc_semantic::Reference,
    semantic: &Semantic<'_>,
) -> bool {
    reference.symbol_id().is_some_and(|id| {
        let flags = semantic.scoping().symbol_flags(id);
        !flags.is_ambient() && !flags.is_type_import()
    })
}

pub(super) fn unwrapped_parent<'a>(
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'a>,
) -> Option<AstKind<'a>> {
    semantic.nodes().ancestor_kinds(node_id).find(|kind| {
        !matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    })
}
