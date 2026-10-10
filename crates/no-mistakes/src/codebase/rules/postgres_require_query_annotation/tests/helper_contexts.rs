use super::*;

#[test]
fn helper_review_regressions_preserve_call_identity_exports_and_live_captures() {
    let root = fixture("helper-tracing-review");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let paths = [
        "src/destructured-tag.mts",
        "src/import-equals.mts",
        "src/local-tag.mts",
        "src/named-tags.mts",
        "src/query.mts",
        "src/raw-assertion.ts",
        "src/raw-deleted.mts",
        "src/raw-reassigned.mts",
        "src/raw-shadowed.mts",
    ];
    for ignore in [false, true] {
        let policy = if ignore { "ignore" } else { "report" };
        let config = config_with_options(&format!(
            "importSpecifier: '@app/db'\ninclude: ['src/query.mts', 'src/import-equals.mts', 'src/destructured-tag.mts', 'src/local-tag.mts', 'src/named-tags.mts', 'src/raw-assertion.ts', 'src/raw-deleted.mts', 'src/raw-reassigned.mts', 'src/raw-shadowed.mts']\ntrustedSqlTags: [{{module: './tags.mjs', name: customQuery}}]\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        for threads in [1, 3] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let parallel = pool.install(|| check_with_files(&root, &config, &files).unwrap());
            assert_eq!(
                parallel, findings,
                "helper projection differs with {threads} workers"
            );
        }
        let expected = paths
            .iter()
            .flat_map(|path| {
                std::fs::read_to_string(root.join(path))
                    .unwrap()
                    .lines()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        (line.contains("// finding:")
                            || (!ignore && line.contains("// unanalyzable:")))
                        .then_some((path.to_string(), index + 1))
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.file.clone(), finding.line))
                .collect::<Vec<_>>(),
            expected,
            "{findings:#?}"
        );
    }
}

#[test]
fn executor_free_root_still_reports_imported_executor_through_cyclic_reexports() {
    let root = fixture("helper-tracing-reachability");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = config_with_options(
        "importSpecifier: '@app/db'\ninclude: ['src/helper.mts']\nunanalyzableSql: ignore",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/helper.mts");
    assert_eq!(findings[0].line, 3);
}

#[test]
fn helper_value_contexts_preserve_captures_parameters_and_implicit_arguments() {
    for scenario in [
        "helper-tracing-post-merge",
        "helper-tracing-tag-callback",
        "helper-tracing-alternative-callback",
        "helper-tracing-named-delete",
        "helper-tracing-argument-members",
        "helper-tracing-discarded-values",
        "helper-tracing-call-argument-order",
        "helper-tracing-live-binding",
        "helper-tracing-argument-slot-write",
        "helper-tracing-destructuring-alias",
        "helper-tracing-arm-module",
        "helper-tracing-argument-length",
        "helper-tracing-returned-callbacks",
        "helper-tracing-deleted-slot-callback",
        "helper-tracing-module-sibling",
        "helper-tracing-module-sibling-reverse",
        "helper-tracing-callback-installers",
        "helper-tracing-mapped-scalar",
        "helper-tracing-mapped-formal-callback",
        "helper-tracing-argument-alias-write",
        "helper-tracing-dynamic-delete-slot-write",
        "helper-tracing-callback-revisit",
        "helper-tracing-imported-callback-revisit",
        "helper-tracing-unary-argument-index",
        "helper-tracing-shared-snapshots",
    ] {
        let root = fixture(scenario);
        let files = crate::codebase::ts_source::discover_visible_paths(&root);
        let names = if scenario == "helper-tracing-arm-module" {
            vec!["src/query.mts", "src/state.mts"]
        } else if matches!(
            scenario,
            "helper-tracing-mapped-scalar" | "helper-tracing-callback-revisit"
        ) {
            vec!["src/query.cjs"]
        } else if scenario == "helper-tracing-imported-callback-revisit" {
            vec!["src/query.mts", "src/helper.mts"]
        } else if scenario == "helper-tracing-mapped-formal-callback" {
            vec!["src/query.mts", "src/helper.cjs"]
        } else if matches!(
            scenario,
            "helper-tracing-argument-alias-write" | "helper-tracing-dynamic-delete-slot-write"
        ) {
            vec!["src/query.mts", "src/sloppy.cjs"]
        } else {
            vec!["src/query.mts"]
        };
        let sources = names
            .iter()
            .map(|name| (*name, std::fs::read_to_string(root.join(name)).unwrap()))
            .collect::<Vec<_>>();
        let include = names
            .iter()
            .map(|name| format!("'{name}'"))
            .collect::<Vec<_>>()
            .join(",");
        for policy in ["report", "ignore"] {
            let config = config_with_options(&format!(
                "importSpecifier: '@app/db'\ninclude: [{include}]\nunanalyzableSql: {policy}"
            ));
            let findings = check_with_files(&root, &config, &files).unwrap();
            let expected = sources
                .iter()
                .flat_map(|(name, source)| {
                    source.lines().enumerate().filter_map(move |(index, line)| {
                        (line.contains("// finding:")
                            || (policy == "report" && line.contains("// unanalyzable:")))
                        .then_some((name.to_string(), index + 1))
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(
                findings
                    .iter()
                    .map(|finding| (finding.file.clone(), finding.line))
                    .collect::<Vec<_>>(),
                expected,
                "{findings:#?}"
            );
        }
    }
}

#[test]
fn missing_alternative_binding_stays_unproven() {
    let root = fixture("helper-tracing-missing-alternative-binding");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    for (policy, markers) in [
        ("report", &["// finding:", "// unanalyzable:"][..]),
        ("ignore", &["// finding:"][..]),
    ] {
        let config = config_with_options(&format!(
            "importSpecifier: '@app/db'\ninclude: ['src/query.mts']\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        let expected = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                markers
                    .iter()
                    .any(|marker| line.contains(marker))
                    .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            expected,
            "{policy}: {findings:#?}"
        );
    }
}
