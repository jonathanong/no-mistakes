use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    pub(super) fn loop_binding(&mut self, id: &IdentifierReference<'a>) -> Option<PathOrigin> {
        if !self.take() {
            return None;
        }
        let symbol = self.binding_symbol(id)?;
        let semantic = self.semantic;
        let node = semantic.scoping().symbol_declaration(symbol);
        let AstKind::VariableDeclarator(declaration) = semantic.nodes().kind(node) else {
            return None;
        };
        if !self.immutable_binding(symbol) {
            return None;
        }
        if declaration.init.is_some()
            || !matches!(&declaration.id, BindingPattern::BindingIdentifier(binding)
                if binding.symbol_id.get() == Some(symbol))
        {
            return None;
        }
        let parent = semantic.nodes().parent_id(node);
        let AstKind::ForOfStatement(ForOfStatement {
            left: ForStatementLeft::VariableDeclaration(left),
            body,
            right,
            ..
        }) = semantic.nodes().parent_kind(parent)
        else {
            return None;
        };
        if left.kind != VariableDeclarationKind::Const
            || left.declarations.len() != 1
            || left.declarations[0].span != declaration.span
            || !body.span().contains_inclusive(id.span)
        {
            return None;
        }
        let Expression::ArrayExpression(array) = right.get_inner_expression() else {
            return None;
        };
        if array.elements.is_empty() || array.elements.len() > 16 {
            return None;
        }
        let mut origins = Vec::with_capacity(array.elements.len());
        for element in &array.elements {
            if !self.take() {
                return None;
            }
            let Expression::StringLiteral(value) = element.as_expression()?.get_inner_expression()
            else {
                return None;
            };
            let text = crate::ast::js_literal(value);
            if text
                .split(['/', '\\'])
                .filter(|part| !part.is_empty() && *part != ".")
                .take(33)
                .count()
                > 32
            {
                return None;
            }
            origins.push(PathOrigin::literal(&text));
        }
        Some(PathOrigin::alternatives(origins))
    }
}

#[cfg(test)]
#[path = "loop_bindings/tests.rs"]
mod tests;
