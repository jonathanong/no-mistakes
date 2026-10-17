//! Partition literal dispatch by length to keep each lookup easy to inspect.
// No map lookup or string allocation is needed for built-in metrics.
pub(super) fn core_index(metric: &str) -> Option<usize> {
    match metric.len() {
        0..=12 => short(metric),
        13..=16 => medium(metric),
        17..=19 => long(metric),
        20..=25 => longer(metric),
        _ => longest(metric),
    }
}

#[inline]
fn short(metric: &str) -> Option<usize> {
    Some(match metric {
        "ast.walks" => 1,
        "graph.builds" => 7,
        "graph.edges" => 10,
        "graph.nodes" => 11,
        "graph.reuses" => 12,
        "parse.errors" => 19,
        "parse.files" => 20,
        "source.bytes" => 33,
        "source.reads" => 36,
        _ => return None,
    })
}

#[inline]
fn medium(metric: &str) -> Option<usize> {
    Some(match metric {
        "discovery.roots" => 6,
        "manifest.errors" => 14,
        "manifest.parses" => 15,
        "output.errors" => 17,
        "output.renders" => 18,
        "parse.requests" => 22,
        "source.requests" => 37,
        "traversal.nodes" => 44,
        "traversal.reuses" => 46,
        "ts_facts.files" => 48,
        "workspace.builds" => 49,
        _ => return None,
    })
}

#[inline]
fn long(metric: &str) -> Option<usize> {
    Some(match metric {
        "analysis.requests" => 0,
        "discovery.requests" => 5,
        "manifest.cache_hits" => 13,
        "manifest.requests" => 16,
        "resolver.cache_hits" => 24,
        "resolver.requests" => 28,
        "resolver.resolved" => 29,
        "resolver.unresolved" => 32,
        "source.cache_hits" => 34,
        "source.read_errors" => 35,
        "symbol_index.builds" => 38,
        "symbol_index.reuses" => 39,
        "test_filter.builds" => 40,
        "traversal.requests" => 45,
        _ => return None,
    })
}

#[inline]
fn longer(metric: &str) -> Option<usize> {
    Some(match metric {
        "discovery.cache_hits" => 2,
        "discovery.candidates" => 3,
        "discovery.projections" => 4,
        "graph.candidate_excluded" => 8,
        "graph.candidate_files" => 9,
        "parse.files_after_extract" => 21,
        "query_diagnostics.builds" => 23,
        "resolver.classifications" => 26,
        "resolver.computations" => 27,
        "resolver.unique_keys" => 31,
        "traversal.computations" => 42,
        "traversal.lazy_nodes" => 43,
        "ts_facts.collections" => 47,
        "postgres.annotation.roots" => 56,
        _ => return None,
    })
}

#[inline]
fn longest(metric: &str) -> Option<usize> {
    Some(match metric {
        "postgres.annotation.modules" => 50,
        "postgres.annotation.target_resolutions" => 51,
        "postgres.annotation.reachability_walks" => 52,
        "postgres.annotation.incoming_dense_slots_copied" => 53,
        "postgres.annotation.captured_reader_rebuilds" => 54,
        "postgres.annotation.captured_reader_names_retained" => 55,
        "resolver.classification_requests" => 25,
        "resolver.scope_cache_requests" => 30,
        "test_filter.project_filters" => 41,
        _ => return None,
    })
}
