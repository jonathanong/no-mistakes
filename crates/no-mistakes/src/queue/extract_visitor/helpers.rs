use super::QueueVisitor;
use crate::queue::extract_helpers::literal_expr;
use oxc_ast::ast::{Argument, Expression};

impl QueueVisitor<'_> {
    pub fn is_queue_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.queue_classes.contains(id.name.as_str()))
    }

    pub fn is_worker_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.worker_classes.contains(id.name.as_str()))
    }

    pub fn is_flow_constructor(&self, expr: &Expression<'_>) -> bool {
        matches!(expr, Expression::Identifier(id) if self.flow_classes.contains(id.name.as_str()))
    }

    pub(super) fn stamp_worker(&mut self, recorded: usize) {
        let Some(index) = self.worker_stack.last().copied() else {
            return;
        };
        for producer in &mut self.facts.producers[recorded..] {
            producer.worker_index = Some(index);
        }
    }

    pub fn literal_arg(&self, arg: &Argument<'_>) -> Option<String> {
        match arg {
            Argument::StringLiteral(value) => Some(value.value.as_str().to_string()),
            Argument::Identifier(id) => self.const_strings.get(id.name.as_str()).cloned(),
            _ => arg
                .as_expression()
                .and_then(|expr| literal_expr(expr, &self.const_strings)),
        }
    }
}
