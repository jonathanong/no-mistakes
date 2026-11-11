use super::super::{ArrayGuard, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::{
    node::NodeId,
    operator::{BinaryOperator, LogicalOperator},
};

pub(super) fn collect<'a>(projector: &mut Projector<'a, '_>, node: NodeId) -> Vec<ArrayGuard> {
    let semantic = projector.semantic;
    let span = semantic.nodes().kind(node).span();
    let mut guards = Vec::new();
    for ancestor in semantic.nodes().ancestors(node).take(32) {
        match ancestor.kind() {
            AstKind::IfStatement(branch)
                if branch.consequent.span().start <= span.start
                    && span.end <= branch.consequent.span().end =>
            {
                condition(projector, &branch.test, &mut guards, 0);
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                guards.retain(|guard| guard.owner == ancestor.kind().span().start);
                return guards;
            }
            _ => {}
        }
    }
    Vec::new()
}

fn condition<'a>(
    projector: &mut Projector<'a, '_>,
    expr: &Expression<'a>,
    out: &mut Vec<ArrayGuard>,
    depth: usize,
) {
    if depth >= 8 {
        return;
    }
    match expr.get_inner_expression() {
        Expression::LogicalExpression(logic) if logic.operator == LogicalOperator::And => {
            condition(projector, &logic.left, out, depth + 1);
            condition(projector, &logic.right, out, depth + 1);
        }
        Expression::BinaryExpression(binary)
            if binary.operator == BinaryOperator::StrictEquality =>
        {
            for (candidate, literal) in
                [(&binary.left, &binary.right), (&binary.right, &binary.left)]
            {
                let expected = match literal.get_inner_expression() {
                    Expression::StringLiteral(s) => {
                        serde_json::Value::String(crate::ast::js_literal(s).into_owned())
                    }
                    Expression::NumericLiteral(n) => {
                        let Some(n) = serde_json::Number::from_f64(n.value) else {
                            continue;
                        };
                        serde_json::Value::Number(n)
                    }
                    _ => continue,
                };
                if let Some(mut guard) = binding(projector, candidate) {
                    guard.expected = expected;
                    out.push(guard);
                }
            }
        }
        _ => {}
    }
}

fn binding<'a>(projector: &mut Projector<'a, '_>, expr: &Expression<'a>) -> Option<ArrayGuard> {
    let (id, length) = match expr.get_inner_expression() {
        Expression::Identifier(id) => (id.as_ref(), false),
        Expression::StaticMemberExpression(member) if member.property.name == "length" => {
            let Expression::Identifier(id) = member.object.get_inner_expression() else {
                return None;
            };
            (id.as_ref(), true)
        }
        _ => return None,
    };
    let semantic = projector.semantic;
    let symbol = semantic
        .scoping()
        .get_reference(id.reference_id.get()?)
        .symbol_id()?;
    if !projector.immutable_binding(symbol) {
        return None;
    }
    let declaration_node = semantic.scoping().symbol_declaration(symbol);
    let AstKind::VariableDeclarator(declaration) = semantic.nodes().kind(declaration_node) else {
        return None;
    };
    let BindingPattern::ArrayPattern(pattern) = &declaration.id else {
        return None;
    };
    if pattern
        .elements
        .iter()
        .flatten()
        .any(|element| !matches!(element, BindingPattern::BindingIdentifier(_)))
    {
        return None;
    }
    let variable_node = semantic.nodes().parent_id(declaration_node);
    let AstKind::VariableDeclaration(variables) = semantic.nodes().kind(variable_node) else {
        return None;
    };
    let AstKind::FunctionBody(body) = semantic.nodes().parent_kind(variable_node) else {
        return None;
    };
    if variables.declarations.len() != 1 || body.statements.first()?.span() != variables.span {
        return None;
    }
    let Expression::Identifier(parameter) = declaration.init.as_ref()?.get_inner_expression()
    else {
        return None;
    };
    let parameter_symbol = semantic
        .scoping()
        .get_reference(parameter.reference_id.get()?)
        .symbol_id()?;
    if semantic
        .scoping()
        .get_resolved_references(parameter_symbol)
        .count()
        != 1
    {
        return None;
    }
    let function = semantic
        .nodes()
        .parent_node(semantic.nodes().parent_id(variable_node));
    let params = match function.kind() {
        AstKind::Function(function) => &function.params,
        AstKind::ArrowFunctionExpression(function) => &function.params,
        _ => return None,
    };
    if params.rest.is_some()
        || params.items.len() != 1
        || params.items[0].initializer.is_some()
        || !matches!(
            params.items[0].pattern,
            BindingPattern::BindingIdentifier(_)
        )
    {
        return None;
    }
    let origin = projector.param_origin(parameter)?;
    if origin.param_member.is_some() {
        return None;
    }
    let index = if length {
        let BindingPattern::BindingIdentifier(rest) = &pattern.rest.as_ref()?.argument else {
            return None;
        };
        if rest.symbol_id.get() != Some(symbol) || !length_reads_only(semantic, symbol) {
            return None;
        }
        pattern.elements.len()
    } else {
        pattern.elements.iter().position(|element| matches!(element, Some(BindingPattern::BindingIdentifier(binding)) if binding.symbol_id.get() == Some(symbol)))?
    };
    Some(ArrayGuard {
        owner: origin.param_owner?,
        parameter: origin.param?,
        index,
        length,
        expected: serde_json::Value::Null,
    })
}

fn length_reads_only(
    semantic: &oxc_semantic::Semantic<'_>,
    symbol: oxc_semantic::SymbolId,
) -> bool {
    semantic.scoping().get_resolved_references(symbol).all(|reference| {
        let nodes = semantic.nodes();
        let member = nodes.parent_id(reference.node_id());
        match nodes.kind(member) {
            AstKind::StaticMemberExpression(property) if property.property.name == "length" => {
                matches!(nodes.parent_kind(member), AstKind::BinaryExpression(binary) if binary.operator == BinaryOperator::StrictEquality)
            }
            AstKind::ComputedMemberExpression(_) => {
                let mut parent = nodes.parent_id(member);
                if matches!(nodes.kind(parent), AstKind::TSNonNullExpression(_)) { parent = nodes.parent_id(parent); }
                matches!(nodes.kind(parent), AstKind::CallExpression(call) if call.arguments.iter().any(|arg| arg.span().start <= nodes.kind(member).span().start && nodes.kind(member).span().end <= arg.span().end))
            }
            _ => false,
        }
    })
}
