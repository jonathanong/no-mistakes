use oxc_ast::ast::JSXText;
use oxc_ast_visit::Visit;
use std::{path::Path, sync::Arc};

#[derive(Default)]
struct Collector {
    ranges: Vec<(usize, usize)>,
}

impl<'a> Visit<'a> for Collector {
    fn visit_jsx_text(&mut self, text: &JSXText<'a>) {
        self.ranges
            .push((text.span.start as usize, text.span.end as usize));
    }
}

pub(crate) fn collect_from_program(program: &oxc_ast::ast::Program<'_>) -> Vec<(usize, usize)> {
    let mut collector = Collector::default();
    collector.visit_program(program);
    collector.ranges
}

pub(crate) fn collect(
    path: &Path,
    content: &Arc<str>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<(usize, usize)> {
    if !super::super::is_jsx_file(path) {
        return Vec::new();
    }
    session
        .with_recovered_program_status(path, content, |program, _, _, _| {
            collect_from_program(program)
        })
        .unwrap_or_default()
}

pub(super) fn mask(content: &str, ranges: &[(usize, usize)]) -> String {
    let mut bytes = content.as_bytes().to_vec();
    for &(start, end) in ranges {
        for byte in &mut bytes[start..end] {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(bytes).expect("masking JSX text preserves UTF-8")
}
