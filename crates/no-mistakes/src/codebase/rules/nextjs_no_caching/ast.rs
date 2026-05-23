use super::visitor::NextjsCachingVisitor;
use super::NextjsCachingFinding;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

pub(crate) fn extract_program(source: &str, program: &Program<'_>) -> Vec<NextjsCachingFinding> {
    let mut findings = Vec::new();
    for directive in &program.directives {
        if is_cache_directive(directive.directive.as_str()) {
            findings.push(NextjsCachingFinding {
                line: byte_offset_to_line(source, directive.span.start as usize) as usize,
                message: "Next.js cache directives are disabled; remove this directive".to_string(),
            });
        }
    }

    let bindings = super::bindings::top_level_bindings(program);
    let mut visitor = NextjsCachingVisitor::new(source, findings, bindings);
    visitor.visit_program(program);
    visitor.findings.sort();
    visitor.findings.dedup();
    visitor.findings
}

pub(super) fn is_cache_directive(value: &str) -> bool {
    matches!(
        value,
        "use cache" | "use cache: private" | "use cache: remote"
    )
}
