fn helper_ref_summary(callee: String) -> RouteHelperContextSummary {
    RouteHelperContextSummary {
        patterns: vec![ROUTE_HELPER_REF_PATTERN_MARKER.to_string()],
        refs: vec![RouteHelperRefCandidate {
            callee,
            wrapper_pattern: ROUTE_HELPER_REF_PATTERN_MARKER.to_string(),
        }],
    }
}

fn dynamic_route_helper_context_summary() -> RouteHelperContextSummary {
    RouteHelperContextSummary {
        patterns: vec!["*".to_string()],
        refs: Vec::new(),
    }
}

fn concat_route_helper_context_summaries(
    left: RouteHelperContextSummary,
    right: RouteHelperContextSummary,
) -> RouteHelperContextSummary {
    let patterns = concat_candidates(&left.patterns, &right.patterns);
    let mut refs = Vec::new();
    for candidate in left.refs {
        for wrapper_pattern in concat_candidates(
            std::slice::from_ref(&candidate.wrapper_pattern),
            &right.patterns,
        ) {
            refs.push(RouteHelperRefCandidate {
                callee: candidate.callee.clone(),
                wrapper_pattern,
            });
        }
    }
    for candidate in right.refs {
        for wrapper_pattern in concat_candidates(
            &left.patterns,
            std::slice::from_ref(&candidate.wrapper_pattern),
        ) {
            refs.push(RouteHelperRefCandidate {
                callee: candidate.callee.clone(),
                wrapper_pattern,
            });
        }
    }
    RouteHelperContextSummary { patterns, refs }
}

fn merge_route_helper_context_summaries(
    mut left: RouteHelperContextSummary,
    right: RouteHelperContextSummary,
) -> RouteHelperContextSummary {
    left.patterns.extend(right.patterns);
    left.patterns = dedupe_candidates(left.patterns);
    left.refs.extend(right.refs);
    left.refs.sort_by(|a, b| {
        (&a.callee, &a.wrapper_pattern).cmp(&(&b.callee, &b.wrapper_pattern))
    });
    left.refs
        .dedup_by(|a, b| a.callee == b.callee && a.wrapper_pattern == b.wrapper_pattern);
    left
}

