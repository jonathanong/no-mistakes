//! Fixed-size, allocation-free helper order accounting; no production telemetry.
use no_mistakes::benchmark_support::{
    collect_language_frontend_edges, collect_language_frontend_facts,
    match_language_frontend_queue_globs, LanguageFrontendFixture, LanguageFrontendSummary,
};

#[derive(Default)]
pub(super) struct Trace {
    events: [&'static str; 12],
    length: usize,
}

impl Trace {
    pub(super) fn push(&mut self, event: &'static str) {
        self.events[self.length] = event;
        self.length += 1;
    }

    pub(super) fn events(&self) -> &[&'static str] {
        &self.events[..self.length]
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Prefix {
    facts: LanguageFrontendSummary,
    pub(super) edges: LanguageFrontendSummary,
    pub(super) globs: LanguageFrontendSummary,
    conditioned: [LanguageFrontendSummary; 5],
}

pub(super) fn recorded_prefix(fixture: &LanguageFrontendFixture, trace: &mut Trace) -> Prefix {
    // This exact helper order reproduces the original CodSpeed extract prefix:
    // facts + edges (one nested facts collection) + glob + five facts warmups.
    trace.push("facts");
    let facts = collect_language_frontend_facts(fixture);
    trace.push("edges");
    let edges = collect_language_frontend_edges(fixture);
    trace.push("globs");
    let globs = match_language_frontend_queue_globs(fixture);
    let conditioned = std::array::from_fn(|_| {
        trace.push("conditioning");
        collect_language_frontend_facts(fixture)
    });
    Prefix {
        facts,
        edges,
        globs,
        conditioned,
    }
}

pub(super) fn validate_prefix(prefix: Prefix) {
    assert_eq!(prefix.facts, expected_facts());
    for facts in prefix.conditioned {
        assert_eq!(facts, expected_facts());
    }
    validate_relationships(prefix.edges, prefix.globs);
}

pub(super) fn validate_relationships(
    edges: LanguageFrontendSummary,
    globs: LanguageFrontendSummary,
) {
    assert_eq!(
        edges,
        LanguageFrontendSummary {
            files: 118,
            edges: 131,
            queue_edges: 14,
            ..LanguageFrontendSummary::default()
        }
    );
    assert_eq!(
        globs,
        LanguageFrontendSummary {
            files: 118,
            glob_matches: 118,
            ..LanguageFrontendSummary::default()
        }
    );
}

pub(super) fn expected_facts() -> LanguageFrontendSummary {
    LanguageFrontendSummary {
        files: 118,
        parsed_files: 70,
        imports: 67,
        enqueues: 7,
        workers: 6,
        route_handlers: 43,
        ..LanguageFrontendSummary::default()
    }
}
