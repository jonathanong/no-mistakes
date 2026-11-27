use super::Projector;
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn argument_expression<'b>(
        &mut self,
        call: &'b CallExpression<'a>,
        index: usize,
    ) -> Option<&'b Expression<'a>>
    where
        'a: 'b,
    {
        let mut position = 0;
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                if position == index {
                    return Some(expression);
                }
                position += 1;
            } else if let Argument::SpreadElement(spread) = argument {
                let expressions = self.positional_elements(&spread.argument, 0)?;
                if let Some(expression) =
                    index.checked_sub(position).and_then(|i| expressions.get(i))
                {
                    return Some(*expression);
                }
                position += expressions.len();
            }
        }
        None
    }

    fn positional_elements<'b>(
        &mut self,
        expression: &'b Expression<'a>,
        depth: usize,
    ) -> Option<Vec<&'b Expression<'a>>>
    where
        'a: 'b,
    {
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
                self.positional_elements(init, depth + 1)
            }
            Expression::ArrayExpression(array) if array.elements.len() <= 32 => {
                let mut expressions = Vec::new();
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => expressions
                            .extend(self.positional_elements(&spread.argument, depth + 1)?),
                        _ => expressions.push(element.as_expression()?),
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
}
