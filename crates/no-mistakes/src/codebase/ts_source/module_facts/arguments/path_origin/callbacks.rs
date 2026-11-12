use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::NodeId;
use oxc_span::GetSpan;
mod classification;
mod lifecycle;
mod roots;

impl<'a> Projector<'a, '_> {
    pub(super) fn callback_origin(&mut self, owner: NodeId, index: usize) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let nodes = semantic.nodes();
        let asynchronous = match nodes.kind(owner) {
            AstKind::ArrowFunctionExpression(function) => {
                function.r#async
                    || match &function.body {
                        ArrowFunctionBody::FunctionBody(body) => !synchronous_body(body),
                        _ => true,
                    }
            }
            AstKind::Function(function) if !function.generator => {
                function.r#async || !synchronous_body(function.body.as_deref()?)
            }
            _ => return None,
        };
        let parent = nodes.parent_id(owner);
        let AstKind::CallExpression(supply) = nodes.kind(parent) else {
            return None;
        };
        let position = supply.arguments.iter().position(|argument| {
            argument
                .as_expression()
                .is_some_and(|value| value.span() == nodes.kind(owner).span())
        })?;
        if supply
            .arguments
            .iter()
            .any(|argument| argument.as_expression().is_none())
        {
            return None;
        }
        let (kind, _) = self.getter_kind(supply)?;
        self.callback_invocations(kind, position, index, asynchronous)
            .or_else(|| {
                self.callback_temp_argument(kind, position, index)
                    .then(PathOrigin::unknown)
            })
    }

    fn callback_invocations(
        &mut self,
        kind: AstKind<'a>,
        position: usize,
        index: usize,
        asynchronous: bool,
    ) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let nodes = semantic.nodes();
        let (helper, params) = match kind {
            AstKind::Function(function) if !function.generator => (function.span, &function.params),
            AstKind::VariableDeclarator(declaration) => {
                match declaration.init.as_ref()?.get_inner_expression() {
                    Expression::ArrowFunctionExpression(function) => {
                        (function.span, &function.params)
                    }
                    Expression::FunctionExpression(function) if !function.generator => {
                        (function.span, &function.params)
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        let parameter = params.items.get(position)?;
        if parameter.initializer.is_some() {
            return None;
        }
        let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
            return None;
        };
        let symbol = binding.symbol_id.get()?;
        let mut invoked = false;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.is_write() {
                return None;
            }
            if reference.is_type() {
                continue;
            }
            let node = reference.node_id();
            let call_node = nodes.parent_id(node);
            let AstKind::CallExpression(call) = nodes.kind(call_node) else {
                return None;
            };
            if call.callee.span() != nodes.kind(node).span() || call.optional {
                return None;
            }
            let enclosing = nodes.ancestors(call_node).find(|ancestor| {
                matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })?;
            if enclosing.kind().span() != helper {
                return None;
            }
            if asynchronous && !matches!(nodes.parent_kind(call_node), AstKind::AwaitExpression(_))
            {
                return None;
            }
            let argument = callback_argument(call, index)?;
            if !self.callback_root(argument, call_node, symbol, helper.start)? {
                return None;
            }
            invoked = true;
        }
        invoked.then(|| PathOrigin::anchored(PathAnchor::Temp))
    }
}

fn callback_argument<'a, 'b>(
    call: &'b CallExpression<'a>,
    index: usize,
) -> Option<&'b Expression<'a>> {
    if call
        .arguments
        .iter()
        .take(index)
        .any(|argument| argument.as_expression().is_none())
    {
        return None;
    }
    call.arguments.get(index)?.as_expression()
}

fn synchronous_body(body: &FunctionBody<'_>) -> bool {
    body.statements
        .iter()
        .all(|statement| synchronous_statement(statement, 0))
}

fn synchronous_statement(statement: &Statement<'_>, depth: usize) -> bool {
    if depth >= 16 {
        return false;
    }
    match statement {
        Statement::ExpressionStatement(_)
        | Statement::VariableDeclaration(_)
        | Statement::EmptyStatement(_) => true,
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .all(|statement| synchronous_statement(statement, depth + 1)),
        Statement::ForOfStatement(looped) => {
            !looped.r#await && synchronous_statement(&looped.body, depth + 1)
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "callbacks/completion_tests.rs"]
mod completion_tests;
