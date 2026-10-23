use super::CheckProgramVisitor;
use crate::codebase::rules::nextjs_no_caching::NextjsCachingVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, BlockStatement, CatchClause, Class, ForInStatement, ForOfStatement,
    ForStatement, Function, FunctionBody, StaticBlock, SwitchStatement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl<'a> CheckProgramVisitor<'a> {
    fn with_nextjs_scope(
        &mut self,
        open: impl FnOnce(&mut NextjsCachingVisitor<'a>),
        visit: impl FnOnce(&mut Self),
    ) {
        if let Some(nextjs) = &mut self.nextjs {
            open(nextjs);
        }
        visit(self);
        if let Some(nextjs) = &mut self.nextjs {
            nextjs.pop_const_scope();
        }
    }

    pub(super) fn walk_function_scope(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.with_nextjs_scope(
            |nextjs| {
                nextjs.open_function_scope(
                    &function.params,
                    function.id.as_ref().map(|id| id.name.as_str()),
                );
            },
            |this| walk::walk_function(this, function, flags),
        );
    }

    pub(super) fn walk_arrow_scope(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_function_scope(&arrow.params, None),
            |this| walk::walk_arrow_function_expression(this, arrow),
        );
    }

    pub(super) fn walk_function_body_scope(&mut self, body: &FunctionBody<'a>) {
        self.with_nextjs_scope(
            |nextjs| {
                nextjs.open_function_body_scope(body);
                nextjs.check_function_body_directives(body);
            },
            |this| walk::walk_function_body(this, body),
        );
    }

    pub(super) fn walk_block_scope(&mut self, block: &BlockStatement<'a>) {
        self.with_nextjs_scope(
            |nextjs| {
                nextjs.push_const_scope();
                nextjs.precollect_statements(&block.body);
            },
            |this| walk::walk_block_statement(this, block),
        );
    }

    pub(super) fn walk_catch_scope(&mut self, catch: &CatchClause<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_catch_scope(catch),
            |this| walk::walk_catch_clause(this, catch),
        );
    }

    pub(super) fn walk_class_scope(&mut self, class: &Class<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_class_scope(class),
            |this| walk::walk_class(this, class),
        );
    }

    pub(super) fn walk_static_scope(&mut self, block: &StaticBlock<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_static_scope(block),
            |this| walk::walk_static_block(this, block),
        );
    }

    pub(super) fn walk_for_scope(&mut self, statement: &ForStatement<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_for_scope(statement),
            |this| walk::walk_for_statement(this, statement),
        );
    }

    pub(super) fn walk_for_in_scope(&mut self, statement: &ForInStatement<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_for_left_scope(&statement.left),
            |this| walk::walk_for_in_statement(this, statement),
        );
    }

    pub(super) fn walk_for_of_scope(&mut self, statement: &ForOfStatement<'a>) {
        self.with_nextjs_scope(
            |nextjs| nextjs.open_for_left_scope(&statement.left),
            |this| walk::walk_for_of_statement(this, statement),
        );
    }

    pub(super) fn walk_switch_scope(&mut self, statement: &SwitchStatement<'a>) {
        // Other fact families must also visit the discriminant exactly once, before cases.
        self.visit_expression(&statement.discriminant);
        self.with_nextjs_scope(
            |nextjs| nextjs.open_switch_scope(statement),
            |this| this.visit_switch_cases(&statement.cases),
        );
    }
}
