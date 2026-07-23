use super::*;

fn resolve_tsconfig(root: &Path, explicit: Option<&Path>) -> Result<TsConfig> {
    resolve_tsconfig_from_visible(
        root,
        explicit,
        &crate::codebase::ts_source::discover_visible_paths(root),
    )
}

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn gitignore_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

#[test]
fn flow_query_covers_deps_and_dependents_directions() {
    let root = fixture_root("tests-impact-symbol");
    for direction in [FlowDirection::Deps, FlowDirection::Dependents] {
        let report = run(&FlowOptions {
            target: "utils.mts#parseDate".to_string(),
            root: root.clone(),
            tsconfig: None,
            config: None,
            direction,
            depth: 1,
            relationships: vec![RelationshipArg::Import],
        })
        .unwrap();

        assert_eq!(report.target, "utils.mts#parseDate");
        assert!(report
            .nodes
            .iter()
            .any(|node| node.id == "utils.mts#parseDate"));
    }
}

#[test]
fn flow_query_symbol_dependents_skip_owner_file_bridge() {
    let root = fixture_root("tests-impact-symbol");
    let report = run(&FlowOptions {
        target: "utils.mts#parseDate".to_string(),
        root,
        tsconfig: None,
        config: None,
        direction: FlowDirection::Dependents,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    })
    .unwrap();

    assert!(report
        .nodes
        .iter()
        .any(|node| node.id == "utils.mts#parseDate"));
    assert!(!report.nodes.iter().any(|node| node.id == "utils.mts"));
}

#[test]
fn flow_ignores_automatic_ignored_tsconfig_but_honors_explicit_path() {
    let fixture = gitignore_fixture();
    let options = |tsconfig| FlowOptions {
        target: "entry.ts".to_string(),
        root: fixture.path().to_path_buf(),
        tsconfig,
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    };

    let automatic = run(&options(None)).unwrap();
    assert!(automatic
        .nodes
        .iter()
        .any(|node| { node.kind == "module" && node.module.as_deref() == Some("@lib/forbidden") }));
    assert!(!automatic
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("src/forbidden.ts")));

    let explicit = run(&options(Some(PathBuf::from("tsconfig.json")))).unwrap();
    assert!(explicit
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("src/forbidden.ts")));
}

#[test]
fn flow_query_deps_edges_and_resolvers_cover_path_branches() {
    let root = fixture_root("tests-impact-symbol");
    let report = run(&FlowOptions {
        target: "other.mts".to_string(),
        root: root.clone(),
        tsconfig: None,
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    })
    .unwrap();

    assert!(report.edges.iter().any(|edge| {
        edge.from == "other.mts" && edge.to == "utils.mts" && edge.kind == "import"
    }));
    assert_eq!(
        resolve_target(&root, root.join("other.mts").to_str().unwrap()),
        NodeId::File(root.join("other.mts"))
    );
    assert_eq!(
        resolve_target(&root, ".github/workflows/main.yml#job:build"),
        NodeId::WorkflowJob {
            workflow_file: root.join(".github/workflows/main.yml"),
            job: "build".to_string(),
        }
    );
    assert_eq!(
        resolve_target(&root, ".github/workflows/main.yml#job:build/step:2"),
        NodeId::WorkflowStep {
            workflow_file: root.join(".github/workflows/main.yml"),
            job: "build".to_string(),
            step: 2,
        }
    );
    let aliased_root = fixture_root("aliased");
    assert!(resolve_tsconfig(&aliased_root, Some(Path::new("tsconfig.json"))).is_ok());
    assert!(resolve_tsconfig(&aliased_root, None).is_ok());
    assert!(resolve_tsconfig(&root, None).is_ok());

    let no_tsconfig_root = fixture_root("agent-response-location");
    let fallback = resolve_tsconfig(&no_tsconfig_root, None).unwrap();
    assert_eq!(fallback.dir, no_tsconfig_root);
    assert!(fallback.paths.is_empty());

    let bad_tsconfig_root = fixture_root("flow-bad-tsconfig");
    let fallback = resolve_tsconfig(&bad_tsconfig_root, None).unwrap();
    assert_eq!(fallback.dir, bad_tsconfig_root);
    assert!(fallback.paths.is_empty());
}

#[test]
fn flow_query_helper_nodes_cover_module_and_queue_variants() {
    let root = fixture_root("simple");
    let module = flow_node(&NodeId::Module("lodash".to_string()), &root, 2);
    assert_eq!(module.kind, "module");
    assert_eq!(module.module.as_deref(), Some("lodash"));

    let queue = flow_node(
        &NodeId::QueueJob {
            queue_file: root.join("jobs.mts"),
            job: "send".to_string(),
        },
        &root,
        3,
    );
    assert_eq!(queue.kind, "queue-job");
    assert_eq!(queue.queue_file.as_deref(), Some("jobs.mts"));
    assert_eq!(queue.job.as_deref(), Some("send"));

    let workflow_job = flow_node(
        &NodeId::WorkflowJob {
            workflow_file: root.join(".github/workflows/main.yml"),
            job: "build".to_string(),
        },
        &root,
        4,
    );
    assert_eq!(workflow_job.kind, "workflow-job");
    assert_eq!(
        workflow_job.workflow_file.as_deref(),
        Some(".github/workflows/main.yml")
    );
    assert_eq!(workflow_job.job.as_deref(), Some("build"));

    let workflow_step = flow_node(
        &NodeId::WorkflowStep {
            workflow_file: root.join(".github/workflows/main.yml"),
            job: "build".to_string(),
            step: 2,
        },
        &root,
        5,
    );
    assert_eq!(workflow_step.kind, "workflow-step");
    assert_eq!(
        workflow_step.workflow_file.as_deref(),
        Some(".github/workflows/main.yml")
    );
    assert_eq!(workflow_step.job.as_deref(), Some("build"));
    assert_eq!(workflow_step.step, Some(2));
}

#[test]
fn flow_query_explicit_missing_tsconfig_errors() {
    let root = fixture_root("simple");
    let error = resolve_tsconfig(&root, Some(Path::new("missing.tsconfig.json"))).unwrap_err();

    assert!(error.to_string().contains("missing.tsconfig.json"));
}

include!("flow_query_tests/prepared_config.rs");
