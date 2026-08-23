use crate::fetch::types::FetchOccurrence;
use crate::fetch::visitor::FetchVisitor;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_span::Span;

pub(crate) fn collect_fetch_calls_in_file(
    program: &Program<'_>,
    source: &str,
    rel_file: &str,
) -> Vec<(Span, FetchOccurrence)> {
    let mut visitor = FetchVisitor::new(source, rel_file, false, false);
    visitor.visit_program(program);
    visitor
        .fetch_spans
        .into_iter()
        .zip(visitor.fetches)
        .collect()
}

#[cfg(test)]
mod tests;
