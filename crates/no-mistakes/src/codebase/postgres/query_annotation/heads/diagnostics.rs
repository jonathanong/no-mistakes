use super::Span;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;

pub(super) struct Gaps {
    global: bool,
    intervals: Vec<Span>,
}
impl Gaps {
    pub fn new(module: &TypeScriptModuleFacts) -> Self {
        let global = module.diagnostics.iter().any(|diagnostic| {
            // Canonical modules/loads.rs emits this diagnostic over the eval
            // call, but its ownership gap can affect later references too.
            diagnostic.span.is_none()
                || (diagnostic.kind == "unsupported"
                    && diagnostic.message == "Dynamic eval may change binding semantics")
        });
        let mut spans = module
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.span.map(|span| (span.start, span.end)))
            .collect::<Vec<_>>();
        spans.sort_unstable();
        let mut intervals: Vec<Span> = Vec::new();
        for span in spans {
            if let Some(last) = intervals.last_mut() {
                if span.0 <= last.1 {
                    last.1 = last.1.max(span.1);
                    continue;
                }
            }
            intervals.push(span);
        }
        Self { global, intervals }
    }
    pub fn overlaps(&self, span: Span) -> bool {
        let index = self.intervals.partition_point(|other| other.1 <= span.0);
        self.global
            || self
                .intervals
                .get(index)
                .is_some_and(|other| other.0 < span.1)
    }
}
