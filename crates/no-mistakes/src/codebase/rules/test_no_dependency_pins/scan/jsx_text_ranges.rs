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

pub(super) fn collect(file: &str, content: &str) -> Vec<(usize, usize)> {
    if !matches!(
        Path::new(file).extension().and_then(|ext| ext.to_str()),
        Some("tsx" | "jsx")
    ) {
        return Vec::new();
    }
    crate::ast::with_recovered_program_status_observed(
        Path::new(file),
        Arc::from(content),
        || {},
        |program, _, _, _| {
            let mut collector = Collector::default();
            collector.visit_program(program);
            collector.ranges
        },
    )
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
