use super::Projector;
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn argument_object<'b>(
        &mut self,
        expression: &'b Expression<'a>,
        depth: usize,
    ) -> Option<&'b ObjectExpression<'a>> {
        if depth >= 16 || !self.take() {
            return None;
        }
        match expression.get_inner_expression() {
            Expression::ObjectExpression(object) => Some(object),
            Expression::Identifier(id) => {
                let symbol = self.binding_symbol(id)?;
                if !self.safe_uses(symbol, depth + 1) {
                    return None;
                }
                let init = self.const_init(id)?;
                self.argument_object(init, depth + 1)
            }
            _ => None,
        }
    }
}
