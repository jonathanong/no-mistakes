use super::*;

fn setup() -> (PathBuf, NoMistakesConfig, DepGraph) {
    let root = fixture("forbidden-dependencies-globs");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let graph = DepGraph::build_with_plan(
        &root,
        &resolve_tsconfig(&root, None).unwrap(),
        GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();
    (root, config, graph)
}

#[test]
fn production_root_globs_preserve_transitive_import_boundaries_and_exceptions() {
    let (root, config, _) = setup();
    let findings = check(&root, &config, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.file.as_str(), f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        vec![
            ("backend/api/routes/get.mts", "shared/testing/helper.mts"),
            ("backend/services/service.mts", "shared/testing/helper.mts"),
            ("backend/workers/job.mts", "shared/testing/helper.mts"),
            ("ci/check-coverage.mts", "tests/other.mts"),
            ("ci/check-health.mts", "shared/testing/coverage.mts"),
            ("ci/check-health.mts", "shared/testing/helper.mts"),
            ("ci/check-health.mts", "tests/other.mts"),
            ("web/app/page.mts", "shared/testing/helper.mts"),
            // Raw rule evaluation precedes shared source-suppression accounting.
            ("web/suppressed.mts", "shared/testing/helper.mts"),
        ]
    );
    assert!(findings.iter().all(|f| f.line == 1));
    assert!(findings
        .iter()
        .all(|f| f.message.contains(&format!("dependencies '{}'", f.file))));
}

#[test]
fn globs_and_exceptions_reuse_prepared_facts_without_extra_discovery_or_parsing() {
    let (root, config, _) = setup();
    super::prepared_support::assert_prepared_matches_standalone(&root, &config, 9);
}

#[test]
fn selected_glob_roots_intersect_the_prepared_file_universe() {
    let (root, mut config, graph) = setup();
    config.rules[0].exclude.clear();
    let opts = config.rules[0].try_rule_options().unwrap();
    let universe = [
        root.join("backend/api/routes/get.mts"),
        root.join("shared/testing/helper.mts"),
    ]
    .into_iter()
    .collect();
    let findings = check_rule_application(
        &root,
        &config,
        &config.rules[0],
        &opts,
        &graph,
        application_scope(None, Some(&universe), None),
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "backend/api/routes/get.mts");
}

#[test]
fn a_clean_exception_root_still_has_a_real_forbidden_import_edge() {
    let (root, config, graph) = setup();
    let source = "ci/check-coverage-scope.mts";
    let target = "test-helpers/vitest-config/index.mts";
    let entries = graph.deps_of(&[NodeId::file(root.join(source))], None, None);
    assert!(entries
        .iter()
        .any(|entry| entry.node == NodeId::file(root.join(target))));
    let mut opts: Options = config.rules[0].try_rule_options().unwrap();
    let allowed = check_application(&root, &opts, &graph).unwrap();
    assert!(allowed.iter().all(|finding| finding.file != source));
    // Removing the exception proves the root and target were selected by the boundary.
    opts.exceptions.clear();
    let forbidden = check_application(&root, &opts, &graph).unwrap();
    assert!(forbidden
        .iter()
        .any(|finding| { finding.file == source && finding.target.as_deref() == Some(target) }));
}

#[test]
fn invalid_selection_and_exception_globs_have_authored_config_locations() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;
    let (root, _, graph) = setup();
    for (name, line, field) in [
        ("invalid-roots.yml", 5, "roots"),
        ("invalid-exclude-roots.yml", 8, "excludeRoots"),
        ("invalid-exception-root.yml", 9, "exceptions[0].roots"),
        (
            "invalid-exception-target.yml",
            10,
            "exceptions[0].allowedFiles",
        ),
    ] {
        let path = root.join(name);
        let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[path])));
        let anchor = ConfigAnchor::resolve(&root, &config, Some(&sources), &config.rules[0]);
        assert_eq!(anchor.nested_option_item_line("missing", 0, "roots", 0), 2);
        let opts = config.rules[0].try_rule_options().unwrap();
        let findings = check_rule_application(
            &root,
            &config,
            &config.rules[0],
            &opts,
            &graph,
            application_scope(None, None, Some(&sources)),
        )
        .unwrap();
        assert_eq!(findings.len(), 1, "{name}: {findings:?}");
        assert_eq!((findings[0].file.as_str(), findings[0].line), (name, line));
        assert!(findings[0].message.contains(field), "{findings:?}");
    }
}

#[test]
fn existing_file_and_directory_roots_containing_glob_syntax_remain_literal() {
    let (root, _, graph) = setup();
    for path in ["literal[entry.mts", "literal[dir"] {
        for selector in [
            path.to_string(),
            root.join(path).to_string_lossy().into_owned(),
        ] {
            let opts = Options {
                roots: vec![selector],
                forbidden_files: vec!["shared/testing/**".into()],
                ..Default::default()
            };
            let findings = check_application(&root, &opts, &graph).unwrap();
            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(
                findings[0].target.as_deref(),
                Some("shared/testing/helper.mts")
            );
        }
    }
}

#[test]
fn absolute_globs_and_no_match_globs_preserve_root_selection() {
    let (root, config, graph) = setup();
    let mut opts: Options = config.rules[0].try_rule_options().unwrap();
    opts.roots = vec![root
        .join("backend/services/**/*.mts")
        .to_string_lossy()
        .into_owned()];
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "backend/services/service.mts");
    opts.exclude_roots = vec![root.join("backend/services").to_string_lossy().into_owned()];
    assert!(check_application(&root, &opts, &graph).unwrap().is_empty());
    opts.roots = vec!["missing/**/*.mts".into()];
    assert!(check_application(&root, &opts, &graph).unwrap().is_empty());
}

#[test]
fn absolute_globs_outside_the_rule_root_do_not_widen_the_source_scope() {
    let (root, _, graph) = setup();
    let opts = Options {
        roots: vec![root.join("ci/check-*.mts").to_string_lossy().into_owned()],
        forbidden_files: vec!["**".into()],
        ..Default::default()
    };
    // The prepared graph can contain files outside a narrowed rule root.
    let findings = check_application(&root.join("backend"), &opts, &graph).unwrap();
    assert!(findings.is_empty());
}

#[test]
#[cfg(unix)]
fn root_globs_preserve_globset_backslash_escapes() {
    let (root, _, graph) = setup();
    let opts = Options {
        // On Unix a backslash escapes the bracket instead of separating path components.
        roots: vec![r"literal\[*/**/*.mts".into()],
        forbidden_files: vec!["shared/testing/**".into()],
        ..Default::default()
    };
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "literal[dir/entry.mts");
}
