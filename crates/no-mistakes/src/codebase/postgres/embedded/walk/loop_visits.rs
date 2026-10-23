use super::{resolve, ScopeVisitor};
use oxc_ast::ast::ForInStatement;
use oxc_ast_visit::walk;

pub(super) fn for_in_statement<'a>(visitor: &mut ScopeVisitor<'a>, statement: &ForInStatement<'a>) {
    visitor.push_scope();
    resolve::bind_for_statement_left(&statement.left, visitor);
    visitor.with_append_loop(statement.span.start, |visitor| {
        walk::walk_for_in_statement(visitor, statement)
    });
    visitor.pop_scope();
}
