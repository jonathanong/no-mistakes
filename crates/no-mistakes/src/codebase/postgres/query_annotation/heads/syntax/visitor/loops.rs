use super::{walk, AstKind, ScopeFlags, Syntax, Visit};
use oxc_ast::ast::*;

impl Syntax {
    pub(super) fn walk_for_loop(&mut self, value: &ForStatement<'_>) {
        if value
            .test
            .as_ref()
            .is_none_or(super::super::super::loop_carried::test)
            && value
                .update
                .as_ref()
                .is_none_or(super::super::super::loop_carried::update)
        {
            self.loop_candidates
                .extend(super::super::super::loop_carried::candidates(
                    &value.body,
                    value.span,
                ));
        }
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
    pub(super) fn walk_for_in_loop(&mut self, value: &ForInStatement<'_>) {
        if super::super::super::loop_carried::iteration(&value.left, &value.right) {
            self.loop_candidates
                .extend(super::super::super::loop_carried::candidates(
                    &value.body,
                    value.span,
                ));
        }
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
    pub(super) fn walk_for_of_loop(&mut self, value: &ForOfStatement<'_>) {
        if !value.r#await && super::super::super::loop_carried::iteration(&value.left, &value.right)
        {
            self.loop_candidates
                .extend(super::super::super::loop_carried::candidates(
                    &value.body,
                    value.span,
                ));
        }
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
    pub(super) fn walk_while_loop(&mut self, value: &WhileStatement<'_>) {
        if super::super::super::loop_carried::test(&value.test) {
            self.loop_candidates
                .extend(super::super::super::loop_carried::candidates(
                    &value.body,
                    value.span,
                ));
        }
        self.repeating(value.span, |this| walk::walk_while_statement(this, value));
    }
    pub(super) fn walk_do_while_loop(&mut self, value: &DoWhileStatement<'_>) {
        if super::super::super::loop_carried::test(&value.test) {
            self.loop_candidates
                .extend(super::super::super::loop_carried::candidates(
                    &value.body,
                    value.span,
                ));
        }
        self.repeating(value.span, |this| {
            walk::walk_do_while_statement(this, value)
        });
    }
}
