pub(super) struct OrderedPattern {
    negated: bool,
    matcher: GlobMatcher,
}

pub(super) fn compile_ordered_patterns(patterns: &[String]) -> Result<Vec<OrderedPattern>> {
    patterns
        .iter()
        .map(|pattern| {
            let trimmed = pattern.trim();
            let (negated, pattern) = trimmed
                .strip_prefix('!')
                .map_or((false, trimmed), |pattern| (true, pattern.trim()));
            let matcher = GlobBuilder::new(&normalize_trigger_glob(pattern))
                .literal_separator(false)
                .build()?
                .compile_matcher();
            Ok(OrderedPattern { negated, matcher })
        })
        .collect()
}

/// Apply every matching entry in order; the final matching entry decides.
pub(super) fn matches_ordered(patterns: &[OrderedPattern], path: &str) -> bool {
    let mut matched = false;
    for pattern in patterns {
        if pattern.matcher.is_match(path) {
            matched = !pattern.negated;
        }
    }
    matched
}

fn normalize_trigger_glob(pattern: &str) -> String {
    let mut part = pattern.trim().to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.trim().to_string();
    }
    part
}

pub(in crate::tests) fn test_runner(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Dotnet => TestRunner::Dotnet,
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Vitest => TestRunner::Vitest,
        TestFramework::Swift => TestRunner::Swift,
        TestFramework::Python => TestRunner::Python,
        TestFramework::Go => TestRunner::Go,
        TestFramework::Cargo => TestRunner::Cargo,
        TestFramework::Rails => TestRunner::Rails,
        TestFramework::Php => TestRunner::Php,
        TestFramework::Java => TestRunner::Java,
        TestFramework::Kotlin => TestRunner::Kotlin,
        TestFramework::Elixir => TestRunner::Elixir,
        TestFramework::Dart => TestRunner::Dart,
        TestFramework::Jest => TestRunner::Jest,
    }
}
