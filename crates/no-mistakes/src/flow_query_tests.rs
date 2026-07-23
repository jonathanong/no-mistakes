use super::*;

#[path = "flow_query_tests/preparation_errors.rs"]
mod preparation_errors;

#[path = "flow_query_tests/basics.rs"]
mod basics;

#[path = "flow_query_tests/vitest_setup.rs"]
mod vitest_setup;

fn resolve_tsconfig(
    root: &Path,
    explicit: Option<&Path>,
) -> Result<crate::codebase::ts_resolver::TsConfig> {
    crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        explicit,
        root,
        &crate::codebase::ts_source::discover_visible_paths(root),
    )
    .or_else(|error| {
        if explicit.is_some() {
            Err(error)
        } else {
            Ok(crate::codebase::ts_resolver::TsConfig {
                dir: root.to_path_buf(),
                paths_dir: root.to_path_buf(),
                paths: Vec::new(),
                base_url: None,
            })
        }
    })
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
fn flow_prepared_graph_honors_explicit_config_without_nested_discovery() {
    let root = fixture_root("graph-default-route-config");
    let empty_config = fixture_root("graph-empty-route-config").join(".no-mistakes.yml");
    let options = |config| FlowOptions {
        target: "src/client.ts".to_string(),
        root: root.clone(),
        tsconfig: None,
        config,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Route],
    };

    let default = run(&options(None)).unwrap();
    assert!(default
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("backend/api/users.mts")));
    let explicit = run(&options(Some(empty_config))).unwrap();
    assert!(!explicit
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("backend/api/users.mts")));

    let source = include_str!("flow_query.rs");
    let run_body = source
        .split("pub fn run(options: &FlowOptions)")
        .nth(1)
        .and_then(|source| source.split("include!(\"flow_query_traverse.rs\")").next())
        .expect("flow run body");
    assert!(run_body.contains("SharedTraversalContext::prepare_with_framework_plan("));
    assert!(!run_body.contains("VisiblePathSnapshot::new("));
    assert!(!run_body.contains("DepGraph::build"));
}
