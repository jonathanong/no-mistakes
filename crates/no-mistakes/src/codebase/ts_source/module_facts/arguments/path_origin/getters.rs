use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};

impl<'a> Projector<'a, '_> {
    pub(super) fn imported_getter(&self, call: &CallExpression<'a>) -> Option<PathOrigin> {
        if !call.arguments.is_empty() || call.optional || call.span.start < self.call_offset {
            return None;
        }
        let Expression::Identifier(id) = call.callee.get_inner_expression() else {
            return None;
        };
        let declaration = self
            .semantic
            .scoping()
            .symbol_declaration(self.binding_symbol(id)?);
        let AstKind::ImportSpecifier(import) = self.semantic.nodes().kind(declaration) else {
            return None;
        };
        if import.import_kind.is_type() {
            return None;
        }
        Some(PathOrigin {
            getter: Some(call.span.start),
            ..PathOrigin::unknown()
        })
    }
}
