use super::{resolve, ScopeVisitor};
use oxc_ast::ast::ReturnStatement;
use oxc_ast_visit::walk;
use oxc_span::GetSpan;

impl ScopeVisitor<'_> {
    pub(super) fn invalidate_execution_variants(&mut self) {
        let owner = self.function_scopes.last().copied().unwrap_or(0);
        for scope in &mut self.scopes[owner..] {
            for binding in scope.values_mut() {
                binding.variants = None;
            }
        }
    }
}
pub(super) fn returned<'a>(visitor: &mut ScopeVisitor<'a>, statement: &ReturnStatement<'a>) {
    let fragment = statement.argument.as_ref().and_then(|argument| {
        resolve::builder_fragment(argument, visitor).map(|sql| (argument.span().start, sql))
    });
    if let Some((start, sql)) = fragment {
        visitor.push_fragment(
            crate::codebase::ts_source::byte_offset_to_line(visitor.source, start as usize),
            Some(sql),
            statement.argument.as_ref(),
        );
        visitor.suppress_nested_builder_fragments += 1;
        walk::walk_return_statement(visitor, statement);
        visitor.suppress_nested_builder_fragments -= 1;
    } else {
        walk::walk_return_statement(visitor, statement);
    }
    visitor.invalidate_execution_variants();
}
