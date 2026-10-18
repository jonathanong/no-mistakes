use super::*;

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
