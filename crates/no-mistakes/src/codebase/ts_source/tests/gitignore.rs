use super::{
    discover_files, discover_files_from_visible, discover_source_files,
    discover_source_files_from_visible, discover_visible_paths, relative_slash_path,
};

#[test]
fn non_git_discovery_applies_gitignore() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");

    let visible: Vec<String> = discover_visible_paths(dir.path())
        .iter()
        .map(|path| relative_slash_path(dir.path(), path))
        .collect();
    let discovered: Vec<String> = discover_files(dir.path(), &[])
        .iter()
        .map(|path| relative_slash_path(dir.path(), path))
        .collect();

    for files in [&visible, &discovered] {
        assert!(files.contains(&"src/visible.mts".to_string()));
        assert!(files.contains(&".github/workflows/visible.yml".to_string()));
        assert!(!files.contains(&"ignored/trap.mts".to_string()));
        assert!(!files.contains(&".github/workflows/ignored.yml".to_string()));
    }
}

#[test]
fn git_discovery_applies_repository_and_global_excludes() {
    let dir = crate::test_support::materialize_gitignore_fixture("exclude-sources");
    crate::test_support::git_init(dir.path());
    std::fs::copy(
        dir.path().join("git-info-exclude.fixture"),
        dir.path().join(".git/info/exclude"),
    )
    .unwrap();
    crate::test_support::git_config(
        dir.path(),
        "core.excludesFile",
        &dir.path().join("global-gitignore.fixture"),
    );
    crate::test_support::git_add_all(dir.path());

    let files: Vec<String> = discover_visible_paths(dir.path())
        .iter()
        .map(|path| relative_slash_path(dir.path(), path))
        .collect();

    assert!(files.contains(&"src/visible.mts".to_string()));
    assert!(!files.contains(&"src/repository-excluded.mts".to_string()));
    assert!(!files.contains(&"src/global-excluded.mts".to_string()));
}

#[test]
fn pass5a_visible_adapters_preserve_fixture_backed_discovery_output() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");
    let visible = discover_visible_paths(dir.path());

    assert_eq!(
        discover_files_from_visible(dir.path(), &[], &visible),
        discover_files(dir.path(), &[])
    );
    assert_eq!(
        discover_source_files_from_visible(dir.path(), &[], &visible),
        discover_source_files(dir.path(), &[])
    );
}

#[test]
fn pass5a_public_wrappers_create_one_request_snapshot() {
    let cases = [
        (
            include_str!("../../../fetches/pipeline/run.rs"),
            "pub(crate) fn run_with_base_root(",
        ),
        (
            include_str!("../../../queue/graph.rs"),
            "pub fn analyze_project(",
        ),
        (
            include_str!("../../../react_traits/pipeline/run.rs"),
            "pub fn run_analyze(",
        ),
        (
            include_str!("../../../react_traits/pipeline/usages.rs"),
            "pub fn run_usages(",
        ),
        (
            include_str!("../../../react_traits/pipeline/check.rs"),
            "pub fn run_check(",
        ),
        (
            include_str!("../../../react_traits/pipeline/check.rs"),
            "pub fn check_enabled(",
        ),
        (include_str!("../../../data_pw_query.rs"), "pub fn run("),
        (include_str!("../../../ci.rs"), "pub fn impact_report("),
        (include_str!("../../../ci.rs"), "pub fn env_report("),
        (
            include_str!("../../ci_graph/mod.rs"),
            "pub fn load(root: &Path, ci: &CiConfig)",
        ),
        (
            include_str!("../../../integration_tests/standalone.rs"),
            "pub(super) fn check(",
        ),
        (
            include_str!("../../../integration_tests.rs"),
            "pub fn check_with_facts(",
        ),
        (
            concat!(
                include_str!("../../../server_routes/graph.rs"),
                include_str!("../../../server_routes/graph_prepare.rs")
            ),
            "pub fn prepare_analysis(",
        ),
    ];

    for (source, signature) in cases {
        let body = function_body(source, signature);
        assert_eq!(
            body.matches("VisiblePathSnapshot::new").count(),
            1,
            "{signature} must create exactly one request snapshot"
        );
    }
}

#[test]
fn pass5a_prepared_bodies_do_not_restart_discovery_or_config_loading() {
    let cases = [
        (
            include_str!("../../../react_traits/pipeline/run.rs"),
            "pub(crate) fn run_analyze_inner_from_visible(",
        ),
        (
            concat!(
                include_str!("../../../react_traits/pipeline/usages.rs"),
                include_str!("../../../react_traits/pipeline/usages_scan.rs")
            ),
            "fn run_usages_from_visible(",
        ),
        (
            include_str!("../../ci_graph/mod.rs"),
            "pub fn load_from_snapshot(",
        ),
        (
            include_str!("../../ci_graph/env_query.rs"),
            "pub fn analyze_env_from_snapshot(",
        ),
        (
            include_str!("../../../integration_tests.rs"),
            "pub fn check_with_prepared_facts(",
        ),
        (
            include_str!("../../../server_routes/graph.rs"),
            "pub fn analyze_project_with_prepared(",
        ),
        (
            include_str!("../../../server_routes/contracts.rs"),
            "pub fn analyze_contracts_with_prepared(",
        ),
    ];
    let forbidden = [
        "VisiblePathSnapshot::new",
        "discover_visible_paths(",
        "discover_files(",
        "discover_source_files(",
        "load_config(",
        "load_v2_config(",
    ];

    for (source, signature) in cases {
        let body = function_body(source, signature);
        for needle in forbidden {
            assert!(!body.contains(needle), "{signature} must not call {needle}");
        }
    }
}

#[test]
fn pass5a_graph_queries_reuse_one_graph_file_discovery() {
    for (source, signature) in [
        (include_str!("../../../effects_query.rs"), "pub fn run("),
        (include_str!("../../../flow_query.rs"), "pub fn run("),
        (
            concat!(
                include_str!("../../../rsc_callers_query.rs"),
                include_str!("../../../rsc_callers_query/prepare.rs")
            ),
            "pub fn run(",
        ),
    ] {
        let body = function_body(source, signature);
        assert_eq!(
            body.matches("VisiblePathSnapshot::new").count(),
            1,
            "{signature}"
        );
        assert_eq!(
            body.matches("discover_files_from_visible").count(),
            1,
            "{signature}"
        );
        assert_eq!(
            body.matches("GraphFiles::from_files").count(),
            1,
            "{signature}"
        );
        assert!(!body.contains("load_v2_config("), "{signature}");
        assert!(!body.contains("discover_visible_paths("), "{signature}");
    }
}

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature {signature}"));
    let brace = start
        + source[start..]
            .find('{')
            .unwrap_or_else(|| panic!("missing body for {signature}"));
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[brace..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[brace..=brace + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated body for {signature}")
}
