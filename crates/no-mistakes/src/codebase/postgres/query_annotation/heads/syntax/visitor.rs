use super::{head, span, Call, Head, Syntax};
use crate::codebase::postgres::query_annotation::heads::assignments::Context;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::{ast::*, AstKind};
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
    fn visit_expression_statement(&mut self, value: &ExpressionStatement<'a>) {
        if let Some(reference) = super::super::append::reference(&value.expression) {
            self.discarded_appends.insert(reference);
        }
        walk::walk_expression_statement(self, value);
    }
    fn visit_class(&mut self, value: &Class<'a>) {
        let prior = self.current_function.replace(span(value.span));
        walk::walk_class(self, value);
        self.current_function = prior;
    }
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        self.contexts
            .insert(span(value.span), self.current_function);
    }
    fn visit_call_expression(&mut self, value: &CallExpression<'a>) {
        self.append_call(value);
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
                    self.append_declaration(binding, declaration.init.as_ref());
                    let head = declaration.init.as_ref().map_or(Head::Unproven, head);
                    if head.is_constant() {
                        self.constants.insert(span(binding.span), head);
                    }
                }
            }
        } else {
            self.assignments.declaration(value);
        }
        walk::walk_variable_declaration(self, value);
    }
    fn visit_expression_statement(&mut self, value: &ExpressionStatement<'a>) {
        let discarded = match unwrap_ts_wrappers(&value.expression) {
            Expression::AssignmentExpression(value) => Some(span(value.span)),
            _ => None,
        };
        let previous = std::mem::replace(&mut self.assignments.discarded_assignment, discarded);
        walk::walk_expression_statement(self, value);
        self.assignments.discarded_assignment = previous;
    }
    fn visit_simple_assignment_target(&mut self, value: &SimpleAssignmentTarget<'a>) {
        self.assignments.target(value);
        walk::walk_simple_assignment_target(self, value);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        value: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.assignments.unsupported_write(value.binding.span);
        walk::walk_assignment_target_property_identifier(self, value);
    }
    fn visit_assignment_expression(&mut self, value: &AssignmentExpression<'a>) {
        self.assignments.assignment(value);
        walk::walk_assignment_expression(self, value);
    }
    fn visit_function(&mut self, value: &Function<'a>, flags: ScopeFlags) {
        let prior = self.current_function.replace(span(value.span));
        self.context(
            Context {
                function: Some(value.span.start),
                repeated: None,
            },
            |this| {
                walk::walk_function(this, value, flags);
            },
        );
        self.current_function = prior;
    }
    fn visit_arrow_function_expression(&mut self, value: &ArrowFunctionExpression<'a>) {
        let prior = self.current_function.replace(span(value.span));
        self.context(
            Context {
                function: Some(value.span.start),
                repeated: None,
            },
            |this| {
                walk::walk_arrow_function_expression(this, value);
            },
        );
        self.current_function = prior;
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
        let kind = AstKind::ForStatement(self.alloc(value));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &value.scope_id);
        self.visit_span(&value.span);
        // The initializer executes once, retaining any enclosing loop context.
        if let Some(init) = &value.init {
            self.visit_for_statement_init(init);
        }
        self.repeating(value.span, |this| {
            if let Some(test) = &value.test {
                this.visit_expression(test);
            }
            if let Some(update) = &value.update {
                this.visit_expression(update);
            }
            this.visit_statement(&value.body);
        });
        self.leave_scope();
        self.leave_node(kind);
    }
    fn visit_for_in_statement(&mut self, value: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(value));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &value.scope_id);
        self.visit_span(&value.span);
        self.repeating(value.span, |this| {
            this.visit_for_statement_left(&value.left);
        });
        // The iterable executes once; an outer loop can still repeat it.
        self.visit_expression(&value.right);
        self.repeating(value.span, |this| this.visit_statement(&value.body));
        self.leave_scope();
        self.leave_node(kind);
    }
    fn visit_for_of_statement(&mut self, value: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(self.alloc(value));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &value.scope_id);
        self.visit_span(&value.span);
        self.repeating(value.span, |this| {
            this.visit_for_statement_left(&value.left);
        });
        // The iterable executes once; an outer loop can still repeat it.
        self.visit_expression(&value.right);
        self.repeating(value.span, |this| this.visit_statement(&value.body));
        self.leave_scope();
        self.leave_node(kind);
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
