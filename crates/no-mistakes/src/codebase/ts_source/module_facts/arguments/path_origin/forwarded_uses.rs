use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{NodeId, Semantic};
use oxc_span::GetSpan;

pub(super) fn spread_is_current(semantic: &Semantic<'_>, node: NodeId, offset: u32) -> bool {
    matches!(semantic.nodes().parent_kind(node), AstKind::CallExpression(call) if call.span.start == offset)
}

pub(super) fn slot_read_safe(semantic: &Semantic<'_>, node: NodeId) -> bool {
    let nodes = semantic.nodes();
    let span = nodes.kind(node).span();
    match nodes.parent_kind(node) {
        AstKind::BinaryExpression(binary) => matches!(
            binary.operator,
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
        ),
        AstKind::IfStatement(statement) => statement.test.span() == span,
        AstKind::ConditionalExpression(expression) => expression.test.span() == span,
        AstKind::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        AstKind::CallExpression(call) => known_path_read(semantic, call),
        _ => false,
    }
}

fn known_path_read(semantic: &Semantic<'_>, call: &CallExpression<'_>) -> bool {
    let Expression::Identifier(id) = call.callee.get_inner_expression() else {
        return false;
    };
    if id.name != "resolve" {
        return false;
    }
    let Some(symbol) = id
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
    else {
        return false;
    };
    let declaration = semantic.scoping().symbol_declaration(symbol);
    if !matches!(semantic.nodes().kind(declaration), AstKind::ImportSpecifier(specifier) if specifier.imported.name() == "resolve" && !specifier.import_kind.is_type())
    {
        return false;
    }
    semantic.nodes().ancestors(declaration).any(|node| {
        matches!(node.kind(), AstKind::ImportDeclaration(import)
            if matches!(import.source.value.as_str(), Some("node:path" | "path")))
    })
}
