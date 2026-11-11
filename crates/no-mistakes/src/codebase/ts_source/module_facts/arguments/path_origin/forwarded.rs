use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

#[path = "forwarded_uses.rs"]
mod uses;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn argument_origin(
        &mut self,
        call: &CallExpression<'a>,
        index: usize,
    ) -> Option<PathOrigin> {
        let mut position = 0;
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                if position == index {
                    return Some(self.origin(expression, 0));
                }
                position += 1;
            } else if let Argument::SpreadElement(spread) = argument {
                if let Some(expressions) = self.array_elements(&spread.argument, 0) {
                    if let Some(expression) =
                        index.checked_sub(position).and_then(|i| expressions.get(i))
                    {
                        return Some(expression.clone());
                    }
                    position += expressions.len();
                } else {
                    return self.rest_origin(&spread.argument, index.checked_sub(position)?, 0);
                }
            }
        }
        None
    }

    fn array_elements(
        &mut self,
        expression: &Expression<'a>,
        depth: usize,
    ) -> Option<Vec<PathOrigin>> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match expression.get_inner_expression() {
            Expression::Identifier(id) => {
                let symbol = self.binding_symbol(id)?;
                if !self.safe_uses(symbol, depth + 1) {
                    return None;
                }
                let init = self.const_init(id)?;
                self.array_elements(init, depth + 1)
            }
            Expression::ArrayExpression(array) if array.elements.len() <= 32 => {
                let mut expressions = Vec::new();
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            expressions.extend(self.array_elements(&spread.argument, depth + 1)?)
                        }
                        _ => expressions.push(self.origin(element.as_expression()?, depth + 1)),
                    }
                    if expressions.len() > 32 {
                        return None;
                    }
                }
                Some(expressions)
            }
            _ => None,
        }
    }

    fn rest_origin(
        &mut self,
        expression: &Expression<'a>,
        index: usize,
        depth: usize,
    ) -> Option<PathOrigin> {
        if depth >= 16 || !self.take() {
            return None;
        }
        let Expression::Identifier(id) = expression.get_inner_expression() else {
            return None;
        };
        let symbol = self.binding_symbol(id)?;
        let semantic = self.semantic;
        let declaration = semantic.scoping().symbol_declaration(symbol);
        let (owner, params) =
            semantic
                .nodes()
                .ancestors(declaration)
                .find_map(|node| match node.kind() {
                    AstKind::Function(function) => Some((node.id(), &function.params)),
                    AstKind::ArrowFunctionExpression(function) => {
                        Some((node.id(), &function.params))
                    }
                    _ => None,
                })?;
        let rest = params.rest.as_ref()?;
        if !matches!(&rest.rest.argument, BindingPattern::BindingIdentifier(binding) if binding.symbol_id.get() == Some(symbol))
        {
            return None;
        }
        let mut writes = Vec::new();
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return None;
            }
            let node = reference.node_id();
            let reference_owner = semantic.nodes().ancestors(node).find(|node| {
                matches!(
                    node.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })?;
            if reference_owner.id() != owner {
                return None;
            }
            let parent = semantic.nodes().parent_node(node);
            match parent.kind() {
                AstKind::SpreadElement(_) => {
                    if !uses::spread_is_current(semantic, parent.id(), self.call_offset) {
                        return None;
                    }
                }
                AstKind::ComputedMemberExpression(member) => {
                    let Expression::NumericLiteral(position) =
                        member.expression.get_inner_expression()
                    else {
                        return None;
                    };
                    if position.value < 0.0 || position.value.fract() != 0.0 {
                        return None;
                    }
                    if let AstKind::AssignmentExpression(assignment) =
                        semantic.nodes().parent_kind(parent.id())
                    {
                        if assignment.left.span() != member.span
                            || assignment.operator != AssignmentOperator::Assign
                        {
                            return None;
                        }
                        if position.value as usize == index {
                            if uses::later_write_repeats(semantic, parent.id(), self.call_offset) {
                                return None;
                            }
                            if assignment.span.end < self.call_offset {
                                writes.push((semantic.nodes().parent_id(parent.id()), assignment));
                            }
                        }
                    } else if !uses::slot_read_safe(semantic, parent.id()) {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        let start = semantic.nodes().kind(owner).span().start;
        let mut origin = PathOrigin {
            param: Some(params.items.len() + index),
            param_owner: Some(start),
            owner_line: Some(super::super::line_in(semantic.source_text(), start)),
            ..PathOrigin::unknown()
        };
        writes.sort_by_key(|(_, assignment)| assignment.span.start);
        for (node, assignment) in writes {
            let value = self.origin(&assignment.right, depth + 1);
            if unconditional(semantic.nodes(), node, owner) {
                origin = value;
            } else {
                origin = PathOrigin::alternatives(vec![origin, value]);
            }
        }
        Some(origin)
    }
}

fn unconditional(
    nodes: &oxc_semantic::AstNodes<'_>,
    node: oxc_semantic::NodeId,
    owner: oxc_semantic::NodeId,
) -> bool {
    let parent = nodes.parent_id(node);
    if !matches!(nodes.kind(parent), AstKind::ExpressionStatement(_)) {
        return false;
    }
    let mut current = nodes.parent_id(parent);
    loop {
        match nodes.kind(current) {
            AstKind::FunctionBody(_) => return nodes.parent_id(current) == owner,
            AstKind::BlockStatement(_) => current = nodes.parent_id(current),
            _ => return false,
        }
    }
}
