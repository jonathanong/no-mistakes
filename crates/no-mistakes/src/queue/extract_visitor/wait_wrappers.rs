use super::QueueVisitor;
use crate::queue::extract_model::LocalCheck;
use crate::queue::source::line_number;
use oxc_ast::ast::{BindingPattern, Expression, Function, VariableDeclarator};

impl QueueVisitor<'_> {
    pub(super) fn note_wait_function(&mut self, function: &Function<'_>) {
        if function.is_function_declaration() && !function.declare && function.body.is_some() {
            if let Some(id) = &function.id {
                self.note_wait_declaration(id.name.as_str(), id.span.start);
            }
        }
    }

    pub(super) fn note_wait_variable(&mut self, declaration: &VariableDeclarator<'_>) {
        if let (BindingPattern::BindingIdentifier(id), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            if matches!(
                init.get_inner_expression(),
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            ) {
                self.note_wait_declaration(id.name.as_str(), id.span.start);
            }
        }
    }

    fn note_wait_declaration(&mut self, name: &str, start: u32) {
        if self.record_waits
            && self.roles.forbid_enqueue_and_wait_declarations
            && self.roles.is_wait_wrapper(name)
        {
            self.facts.checks.push(LocalCheck {
                line: line_number(self.source, start),
                kind: "enqueue-and-wait-declaration".to_string(),
                job: None,
                message: format!("declaring blocking wrapper `{name}` permits waiting across queues; enqueue without waiting and continue from a completion event"),
            });
        }
    }
}
