//! One original AST syntax collection for requested annotation facts.
use super::Syntax;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

pub(in crate::codebase::postgres::query_annotation::heads) fn collect_with_helpers(
    program: &Program<'_>,
    collect_append_tags: bool,
    helpers: bool,
    collect_helper_heads: bool,
) -> Syntax {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.head_syntax_visits", 1);
    }
    let mut syntax = Syntax {
        collect_append_tags,
        collect_helper_heads,
        collect_finite: collect_helper_heads
            && program.source_text.contains("/*")
            && program.source_text.contains("${"),
        literal_helpers: helpers.then(Vec::new),
        ..Default::default()
    };
    syntax.visit_program(program);
    syntax
}
