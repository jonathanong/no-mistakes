use crate::fx::FxHashMap;
use oxc_ast::{
    ast::{BindingPattern, Expression, Program, TemplateLiteral},
    AstKind,
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_syntax::symbol::SymbolId;

pub(super) fn static_templates(program: &Program<'_>) -> FxHashMap<u32, String> {
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    static_templates_from_semantic(&built.semantic)
}

/// Symbol identity keeps parameter/block shadowing and imports distinct. All
/// consumers borrow this projection of the caller's already parsed program.
pub(crate) fn static_templates_from_semantic(semantic: &Semantic<'_>) -> FxHashMap<u32, String> {
    let values = semantic
        .scoping()
        .symbol_ids()
        .filter_map(|symbol| const_string(semantic, symbol).map(|value| (symbol, value)))
        .collect();
    semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::TemplateLiteral(template) = node.kind() else {
                return None;
            };
            fold_template(semantic, template, &values).map(|text| (template.span.start, text))
        })
        .collect()
}

fn const_string(semantic: &Semantic<'_>, symbol: SymbolId) -> Option<(u32, String)> {
    let scoping = semantic.scoping();
    if !scoping.symbol_flags(symbol).is_const_variable()
        || !scoping.symbol_redeclarations(symbol).is_empty()
        || scoping
            .get_resolved_references(symbol)
            .any(|reference| reference.is_write())
    {
        return None;
    }
    let AstKind::VariableDeclarator(declaration) =
        semantic.nodes().kind(scoping.symbol_declaration(symbol))
    else {
        return None;
    };
    let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
        return None;
    };
    if binding.symbol_id.get() != Some(symbol) {
        return None;
    }
    let value = match &declaration.init {
        Some(Expression::StringLiteral(literal)) => crate::ast::js_literal(literal).into_owned(),
        Some(Expression::TemplateLiteral(template)) if template.expressions.is_empty() => {
            crate::ast::template_literal_text(template, "")
        }
        _ => return None,
    };
    Some((declaration.span.end, value))
}

fn fold_template(
    semantic: &Semantic<'_>,
    template: &TemplateLiteral<'_>,
    values: &FxHashMap<SymbolId, (u32, String)>,
) -> Option<String> {
    if template.expressions.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(template.expressions.len());
    for expression in &template.expressions {
        let Expression::Identifier(identifier) = expression else {
            return None;
        };
        let reference = semantic
            .scoping()
            .get_reference(identifier.reference_id.get()?);
        let (end, value) = values.get(&reference.symbol_id()?)?;
        // Preserve reads before initialization, including TDZ reads.
        if identifier.span.start < *end {
            return None;
        }
        parts.push(value.as_str());
    }
    let mut text = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        text.push_str(&crate::ast::template_quasi_text(quasi));
        if let Some(part) = parts.get(index) {
            text.push_str(part);
        }
    }
    Some(text)
}
