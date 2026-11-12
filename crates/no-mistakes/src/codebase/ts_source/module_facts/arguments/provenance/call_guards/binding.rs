use super::super::super::{ArrayGuard, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{Semantic, SymbolId};
use oxc_span::GetSpan;
use oxc_syntax::{node::NodeId, operator::BinaryOperator};

pub(super) fn guard<'a>(
    projector: &mut Projector<'a, '_>,
    expr: &Expression<'a>,
) -> Option<ArrayGuard> {
    let (id, length) = candidate(expr)?;
    let semantic = projector.semantic;
    let symbol = semantic
        .scoping()
        .get_reference(id.reference_id.get()?)
        .symbol_id()?;
    if !projector.immutable_binding(symbol) {
        return None;
    }
    let (pattern, parameter, variable_node) = first_destructure(semantic, symbol)?;
    let parameter_symbol = semantic
        .scoping()
        .get_reference(parameter.reference_id.get()?)
        .symbol_id()?;
    if semantic
        .scoping()
        .get_resolved_references(parameter_symbol)
        .count()
        != 1
        || !plain_parameter(semantic, variable_node)
    {
        return None;
    }
    let origin = projector.param_origin(parameter)?;
    if origin.param_member.is_some() {
        return None;
    }
    Some(ArrayGuard {
        owner: origin.param_owner?,
        parameter: origin.param?,
        index: selected_index(semantic, pattern, symbol, length)?,
        length,
        expected: serde_json::Value::Null,
    })
}

fn candidate<'s, 'a>(expr: &'s Expression<'a>) -> Option<(&'s IdentifierReference<'a>, bool)> {
    match expr.get_inner_expression() {
        Expression::Identifier(id) => Some((id.as_ref(), false)),
        Expression::StaticMemberExpression(member) if member.property.name == "length" => {
            let Expression::Identifier(id) = member.object.get_inner_expression() else {
                return None;
            };
            Some((id.as_ref(), true))
        }
        _ => None,
    }
}

fn first_destructure<'s, 'a>(
    semantic: &'s Semantic<'a>,
    symbol: SymbolId,
) -> Option<(&'s ArrayPattern<'a>, &'s IdentifierReference<'a>, NodeId)> {
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
    Some((pattern, parameter, variable_node))
}

fn plain_parameter(semantic: &Semantic<'_>, variable_node: NodeId) -> bool {
    let function = semantic
        .nodes()
        .parent_node(semantic.nodes().parent_id(variable_node));
    let params = match function.kind() {
        AstKind::Function(function) => &function.params,
        AstKind::ArrowFunctionExpression(function) => &function.params,
        _ => return false,
    };
    params.rest.is_none()
        && params.items.len() == 1
        && params.items[0].initializer.is_none()
        && matches!(
            params.items[0].pattern,
            BindingPattern::BindingIdentifier(_)
        )
}

fn selected_index(
    semantic: &Semantic<'_>,
    pattern: &ArrayPattern<'_>,
    symbol: SymbolId,
    length: bool,
) -> Option<usize> {
    if !length {
        return pattern.elements.iter().position(|element| matches!(element, Some(BindingPattern::BindingIdentifier(binding)) if binding.symbol_id.get() == Some(symbol)));
    }
    let BindingPattern::BindingIdentifier(rest) = &pattern.rest.as_ref()?.argument else {
        return None;
    };
    (rest.symbol_id.get() == Some(symbol) && length_reads_only(semantic, symbol))
        .then_some(pattern.elements.len())
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
