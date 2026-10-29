//! Bindings and same-file getters that resolve to an owned or env-derived root.
use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};

fn returned<'b, 'a>(
    params: &FormalParameters<'a>,
    body: &'b FunctionBody<'a>,
) -> Option<&'b Expression<'a>> {
    if !params.items.is_empty() || params.rest.is_some() || body.statements.len() != 1 {
        return None;
    }
    match &body.statements[0] {
        Statement::ReturnStatement(statement) => statement.argument.as_ref(),
        _ => None,
    }
}

impl<'a> Projector<'a, '_> {
    /// A `let` whose every write, including its initializer, is an owned temp root.
    pub(super) fn owned_binding(
        &mut self,
        id: &IdentifierReference<'a>,
        depth: usize,
    ) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        if self.exported.contains(&symbol.index()) {
            return None;
        }
        let AstKind::VariableDeclarator(declaration) = semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        else {
            return None;
        };
        if !matches!(declaration.id, BindingPattern::BindingIdentifier(_)) {
            return None;
        }
        let mut writes = usize::from(declaration.init.is_some());
        if let Some(init) = &declaration.init {
            if self.origin(init, depth + 1).anchor != PathAnchor::Temp {
                return None;
            }
        }
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !reference.flags().is_write() {
                continue;
            }
            writes += 1;
            let AstKind::AssignmentExpression(assignment) =
                semantic.nodes().parent_kind(reference.node_id())
            else {
                return None;
            };
            if writes > 16
                || assignment.operator != AssignmentOperator::Assign
                || self.origin(&assignment.right, depth + 1).anchor != PathAnchor::Temp
            {
                return None;
            }
        }
        (writes > 0).then(|| PathOrigin {
            anchor: PathAnchor::Temp,
            dynamic: false,
            ..PathOrigin::unknown()
        })
    }

    /// `getDir()` for a same-file zero-argument function with one returned expression.
    pub(super) fn getter_origin(
        &mut self,
        call: &CallExpression<'a>,
        depth: usize,
    ) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
            return None;
        };
        if !call.arguments.is_empty() {
            return None;
        }
        let symbol = self.binding_symbol(callee)?;
        let expression = match semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        {
            AstKind::Function(function) if !function.generator => {
                returned(&function.params, function.body.as_ref()?)?
            }
            AstKind::VariableDeclarator(declaration) if self.immutable_binding(symbol) => {
                match declaration.init.as_ref()?.get_inner_expression() {
                    Expression::ArrowFunctionExpression(arrow) => {
                        if !arrow.params.items.is_empty() || arrow.params.rest.is_some() {
                            return None;
                        }
                        match &arrow.body {
                            ArrowFunctionBody::FunctionBody(body) => returned(&arrow.params, body)?,
                            other => other.as_expression()?,
                        }
                    }
                    Expression::FunctionExpression(function) if !function.generator => {
                        returned(&function.params, function.body.as_ref()?)?
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        Some(self.origin(expression, depth + 1))
    }
}
