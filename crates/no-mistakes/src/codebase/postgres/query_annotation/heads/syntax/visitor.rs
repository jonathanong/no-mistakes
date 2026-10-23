use super::{head, span, Call, Head, Syntax};
use crate::codebase::postgres::query_annotation::heads::assignments::Context;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl Syntax {
    fn context(&mut self, context: Context, walk: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.assignments.context, context);
        walk(self);
        self.assignments.context = previous;
    }
    fn repeating(&mut self, span: oxc_span::Span, walk: impl FnOnce(&mut Self)) {
        let mut context = self.assignments.context;
        context.repeated.get_or_insert((span.start, span.end));
        self.context(context, walk);
    }
}
impl<'a> Visit<'a> for Syntax {
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        let argument = value.arguments.first().and_then(|arg| arg.as_expression());
        self.calls.push(Call {
            span: span(value.span),
            head: argument.map_or(Head::Unproven, head),
            context: self.assignments.context,
        });
        walk::walk_call_expression(self, value);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declaration in &value.declarations {
                if let BindingPattern::BindingIdentifier(binding) = &declaration.id {
                    let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                    if matches!(
                        head,
                        Head::Literal | Head::Tag | Head::Missing | Head::MissingTag
                    ) {
                        self.constants.insert(span(binding.span), head);
                    }
                }
            }
        } else {
            self.assignments.declaration(value);
        }
        walk::walk_variable_declaration(self, value);
    }
    fn visit_assignment_expression(&mut self, value: &AssignmentExpression<'a>) {
        self.assignments.assignment(value);
        walk::walk_assignment_expression(self, value);
    }
    fn visit_function(&mut self, value: &Function<'a>, flags: ScopeFlags) {
        self.context(
            Context {
                function: Some(value.span.start),
                repeated: None,
            },
            |this| {
                walk::walk_function(this, value, flags);
            },
        );
    }
    fn visit_arrow_function_expression(&mut self, value: &ArrowFunctionExpression<'a>) {
        self.context(
            Context {
                function: Some(value.span.start),
                repeated: None,
            },
            |this| {
                walk::walk_arrow_function_expression(this, value);
            },
        );
    }
    fn visit_property_definition(&mut self, value: &PropertyDefinition<'a>) {
        self.context(Context::default(), |this| {
            walk::walk_property_definition(this, value)
        });
    }
    fn visit_accessor_property(&mut self, value: &AccessorProperty<'a>) {
        self.context(Context::default(), |this| {
            walk::walk_accessor_property(this, value)
        });
    }
    fn visit_static_block(&mut self, value: &StaticBlock<'a>) {
        self.context(Context::default(), |this| {
            walk::walk_static_block(this, value)
        });
    }
    fn visit_for_statement(&mut self, value: &ForStatement<'a>) {
        self.repeating(value.span, |this| walk::walk_for_statement(this, value));
    }
    fn visit_for_in_statement(&mut self, value: &ForInStatement<'a>) {
        self.repeating(value.span, |this| walk::walk_for_in_statement(this, value));
    }
    fn visit_for_of_statement(&mut self, value: &ForOfStatement<'a>) {
        self.repeating(value.span, |this| walk::walk_for_of_statement(this, value));
    }
    fn visit_while_statement(&mut self, value: &WhileStatement<'a>) {
        self.repeating(value.span, |this| walk::walk_while_statement(this, value));
    }
    fn visit_do_while_statement(&mut self, value: &DoWhileStatement<'a>) {
        self.repeating(value.span, |this| {
            walk::walk_do_while_statement(this, value)
        });
    }
    fn visit_for_statement_left(&mut self, value: &ForStatementLeft<'a>) {
        let previous = std::mem::replace(&mut self.assignments.iteration_target, true);
        walk::walk_for_statement_left(self, value);
        self.assignments.iteration_target = previous;
    }
}
