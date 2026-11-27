//! Bindings and same-file getters that resolve to an owned or env-derived root.
use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
#[path = "owned_helpers.rs"]
mod owned_helpers;

fn parameterized(params: &FormalParameters<'_>) -> bool {
    !params.items.is_empty()
}

fn returned<'b, 'a>(
    params: &FormalParameters<'a>,
    body: &'b FunctionBody<'a>,
) -> Option<&'b Expression<'a>> {
    if params.rest.is_some() || body.statements.is_empty() {
        return None;
    }
    if !body.statements[..body.statements.len() - 1]
        .iter()
        .all(|statement| {
            matches!(
                statement,
                Statement::VariableDeclaration(_) | Statement::ExpressionStatement(_)
            )
        })
    {
        return None;
    }
    match body.statements.last()? {
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
        let mut origins = Vec::new();
        let mut writes = usize::from(declaration.init.is_some());
        if let Some(init) = &declaration.init {
            let origin = self.origin(init, depth + 1);
            if !owned(&origin) {
                return None;
            }
            origins.push(origin);
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
            if writes > 16 || assignment.operator != AssignmentOperator::Assign {
                return None;
            }
            let origin = self.origin(&assignment.right, depth + 1);
            if !owned(&origin) {
                return None;
            }
            origins.push(origin);
        }
        (writes > 0).then(|| PathOrigin::alternatives(origins))
    }

    /// `getDir()` / `mkTmp(prefix)` for a same-file function with one returned expression.
    pub(super) fn getter_origin(
        &mut self,
        call: &CallExpression<'a>,
        depth: usize,
    ) -> Option<PathOrigin> {
        if let Some((expression, has_parameters)) = self.getter_expression(call) {
            return super::patterns::accept_parameterized(
                has_parameters,
                self.origin(expression, depth + 1),
            );
        }
        self.owned_helper_origin(call, depth)
    }

    /// The returned expression and whether the callee declares parameters.
    pub(super) fn getter_expression(
        &mut self,
        call: &CallExpression<'a>,
    ) -> Option<(&'a Expression<'a>, bool)> {
        let (kind, symbol) = self.getter_kind(call)?;
        match kind {
            AstKind::Function(function) if !function.generator => {
                let body = function.body.as_ref()?;
                Some((
                    returned(&function.params, body)?,
                    parameterized(&function.params),
                ))
            }
            AstKind::VariableDeclarator(declaration) if self.immutable_binding(symbol) => {
                function_value(declaration.init.as_ref()?)
            }
            _ => None,
        }
    }
    pub(super) fn getter_kind(
        &mut self,
        call: &CallExpression<'a>,
    ) -> Option<(AstKind<'a>, oxc_semantic::SymbolId)> {
        let semantic = self.semantic;
        let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
            return None;
        };
        let symbol = self.binding_symbol(callee)?;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return None;
            }
        }
        Some((
            semantic
                .nodes()
                .kind(semantic.scoping().symbol_declaration(symbol)),
            symbol,
        ))
    }
}

fn function_value<'a>(init: &'a Expression<'a>) -> Option<(&'a Expression<'a>, bool)> {
    match init.get_inner_expression() {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.params.rest.is_some() {
                return None;
            }
            let expression = match &arrow.body {
                ArrowFunctionBody::FunctionBody(body) => returned(&arrow.params, body)?,
                other => other.as_expression()?,
            };
            Some((expression, parameterized(&arrow.params)))
        }
        Expression::FunctionExpression(function) if !function.generator => {
            let body = function.body.as_ref()?;
            Some((
                returned(&function.params, body)?,
                parameterized(&function.params),
            ))
        }
        _ => None,
    }
}

fn owned(origin: &PathOrigin) -> bool {
    origin.atoms().iter().all(|atom| {
        atom.anchor == PathAnchor::Temp
            && atom.root_tails.is_empty()
            && !atom.segments.iter().any(|part| part == "..")
    })
}
