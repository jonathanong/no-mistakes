use super::*;

fn workflow_topology_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/workflow-topology"),
    )
}

fn workflow_node(root: &Path, job: &str) -> NodeId {
    NodeId::WorkflowJob {
        workflow_file: root.join(".github/workflows/main.yml"),
        job: job.to_string(),
    }
}

fn workflow_step(root: &Path, job: &str, step: usize) -> NodeId {
    NodeId::WorkflowStep {
        workflow_file: root.join(".github/workflows/main.yml"),
        job: job.to_string(),
        step,
    }
}

fn graph_has_edge(graph: &DepGraph, from: NodeId, to: NodeId, kind: EdgeKind) -> bool {
    graph
        .edges
        .edges()
        .iter()
        .any(|edge| edge.from == from && edge.to == to && edge.kind == kind)
}

#[test]
fn workflow_virtual_nodes_normalize_display_and_track_their_file_universe() {
    let root = workflow_topology_fixture();
    let workflow_file = root.join(".github/workflows/../workflows/main.yml");
    let normalized_file =
        crate::codebase::ts_resolver::normalize_path(&root.join(".github/workflows/main.yml"));
    let nodes = normalize_nodes(&[
        NodeId::WorkflowJob {
            workflow_file: workflow_file.clone(),
            job: "build".to_string(),
        },
        NodeId::WorkflowStep {
            workflow_file,
            job: "build".to_string(),
            step: 2,
        },
    ]);

    assert_eq!(
        nodes[0].display_name(&root),
        ".github/workflows/main.yml#job:build"
    );
    assert_eq!(
        nodes[1].display_name(&root),
        ".github/workflows/main.yml#job:build/step:2"
    );
    let universe = HashSet::from([normalized_file]);
    assert!(nodes.iter().all(|node| node.is_in_file_universe(&universe)));
}

#[test]
fn workflow_topology_builds_job_step_uses_and_run_edges() {
    let root = workflow_topology_fixture();
    let files = GraphFiles::discover(&root);
    assert!(
        files
            .all()
            .contains(&root.join(".github/workflows/main.yml")),
        "workflow fixture must be part of the graph file universe: {:?}",
        files.all()
    );
    let graph = DepGraph::build_with_plan(&root, &TsConfig::default(), GraphBuildPlan::all())
        .expect("workflow fixture graph");
    let workflow = NodeId::File(root.join(".github/workflows/main.yml"));
    let build = workflow_node(&root, "build");
    let consume = workflow_node(&root, "consume");

    assert!(graph_has_edge(
        &graph,
        workflow,
        build.clone(),
        EdgeKind::WorkflowJob
    ));
    assert!(graph_has_edge(
        &graph,
        build.clone(),
        consume,
        EdgeKind::WorkflowNeeds
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_node(&root, "call"),
        NodeId::File(root.join(".github/workflows/reusable.yml")),
        EdgeKind::WorkflowUses
    ));
    assert!(graph_has_edge(
        &graph,
        build.clone(),
        workflow_step(&root, "build", 0),
        EdgeKind::WorkflowStep
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 0),
        NodeId::File(root.join("scripts/direct.mjs")),
        EdgeKind::WorkflowRun
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 1),
        NodeId::File(root.join("package.json")),
        EdgeKind::WorkflowRun
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 1),
        NodeId::File(root.join("scripts/build.mjs")),
        EdgeKind::WorkflowRun
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 2),
        NodeId::File(root.join("packages/tool/check.mjs")),
        EdgeKind::WorkflowRun
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 3),
        NodeId::File(root.join(".github/actions/local/action.yml")),
        EdgeKind::WorkflowUses
    ));
}

#[test]
fn workflow_artifacts_connect_exact_producer_and_consumer_steps() {
    let root = workflow_topology_fixture();
    let graph = DepGraph::build_with_plan(&root, &TsConfig::default(), GraphBuildPlan::all())
        .expect("workflow fixture graph");

    assert!(graph_has_edge(
        &graph,
        workflow_step(&root, "build", 5),
        workflow_step(&root, "consume", 0),
        EdgeKind::WorkflowArtifact
    ));
}

#[test]
fn workflow_graph_uses_configured_workflow_directories() {
    let root = workflow_topology_fixture();
    let graph = DepGraph::build_with_plan(&root, &TsConfig::default(), GraphBuildPlan::all())
        .expect("configured workflow graph");
    let workflow = root.join("custom/workflows/custom.yml");

    assert!(graph_has_edge(
        &graph,
        NodeId::File(workflow.clone()),
        NodeId::WorkflowJob {
            workflow_file: workflow,
            job: "configured".to_string(),
        },
        EdgeKind::WorkflowJob
    ));
}

#[test]
fn workflow_edges_support_relative_graph_roots() {
    let root = workflow_topology_fixture();
    let current = std::env::current_dir().unwrap();
    let current_parts: Vec<_> = current.components().collect();
    let root_parts: Vec<_> = root.components().collect();
    let common = current_parts
        .iter()
        .zip(&root_parts)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative_root = PathBuf::new();
    for _ in common..current_parts.len() {
        relative_root.push("..");
    }
    for component in &root_parts[common..] {
        relative_root.push(component.as_os_str());
    }
    let graph = DepGraph::build_with_plan(
        &relative_root,
        &TsConfig::default(),
        GraphBuildPlan {
            workflow_topology: true,
            ..GraphBuildPlan::default()
        },
    )
    .expect("relative-root workflow graph");
    let expected_root = crate::codebase::ts_resolver::normalize_path(&relative_root);

    assert!(graph_has_edge(
        &graph,
        workflow_step(&expected_root, "build", 0),
        NodeId::File(expected_root.join("scripts/direct.mjs")),
        EdgeKind::WorkflowRun
    ));
    assert!(graph_has_edge(
        &graph,
        workflow_step(&expected_root, "build", 3),
        NodeId::File(expected_root.join(".github/actions/local/action.yml")),
        EdgeKind::WorkflowUses
    ));
}

#[test]
fn workflow_command_parsing_is_literal_and_conservative() {
    assert_eq!(
        static_command_segments(
            "FOO=bar node 'scripts/direct.mjs'; npm run nested && ./scripts/build.mjs"
        ),
        vec![
            vec!["FOO=bar", "node", "scripts/direct.mjs"],
            vec!["npm", "run", "nested"],
            vec!["./scripts/build.mjs"],
        ]
    );
    assert!(static_command_segments("node x | tee out").is_empty());
    assert!(static_command_segments("node $(dynamic)").is_empty());
    assert!(static_command_segments("cd scripts && node direct.mjs").is_empty());
    assert!(static_command_segments("MODE=test \"cd\" scripts; node direct.mjs").is_empty());
    assert!(static_command_segments("# disabled; node scripts/old.mjs").is_empty());
    assert_eq!(
        static_command_segments("node scripts/direct.mjs;# disabled; node scripts/old.mjs"),
        vec![vec!["node", "scripts/direct.mjs"]]
    );
    assert_eq!(
        static_command_segments("echo ok # && node scripts/old.mjs\nnode scripts/direct.mjs"),
        vec![vec!["echo", "ok"], vec!["node", "scripts/direct.mjs"]]
    );
    assert_eq!(
        static_command_segments(r"node scripts/direct\ file.mjs"),
        vec![vec!["node", "scripts/direct file.mjs"]]
    );
    assert!(static_command_segments("cd scripts # stop").is_empty());
    assert!(static_command_segments("''").is_empty());
    assert_eq!(
        shellish_literal_words(r#""double's quote" escaped\ space"#),
        Some(vec![
            "double's quote".to_string(),
            "escaped space".to_string()
        ])
    );
    assert!(shellish_literal_words("'unterminated").is_none());
    assert!(shellish_literal_words("trailing\\").is_none());
    assert!(!is_static_path_token("${SCRIPT}"));
    assert!(is_environment_assignment("_NAME=value"));
    assert!(!is_environment_assignment("9NAME=value"));
    assert_eq!(
        interpreter_script(&["node".into(), "--".into(), "script.mjs".into()]),
        Some("script.mjs")
    );
    assert_eq!(interpreter_script(&["node".into(), "--check".into()]), None);
    assert_eq!(
        interpreter_script(&[
            "node".into(),
            "--enable-source-maps".into(),
            "script.mjs".into()
        ]),
        Some("script.mjs")
    );
    assert_eq!(
        interpreter_script(&["deno".into(), "run".into(), "script.ts".into()]),
        Some("script.ts")
    );
    assert_eq!(
        interpreter_script(&["deno".into(), "script.ts".into()]),
        None
    );
    assert_eq!(
        interpreter_script(&["python".into(), "-m".into(), "module".into()]),
        None
    );
    assert_eq!(
        package_script_command(&["npm".into(), "test".into()]),
        Some("test")
    );
    assert_eq!(
        package_script_command(&["yarn".into(), "build".into()]),
        Some("build")
    );
    assert_eq!(
        package_script_command(&["pnpm".into(), "build".into()]),
        Some("build")
    );
    assert_eq!(
        package_script_command(&["pnpm".into(), "run".into(), "build".into()]),
        Some("build")
    );
    assert_eq!(
        package_script_command(&["yarn".into(), "run".into(), "build".into()]),
        Some("build")
    );
}

#[test]
fn workflow_run_resolution_handles_cycles_cargo_and_unsafe_inputs() {
    let root = workflow_topology_fixture();
    let direct = root.join("scripts/direct.mjs");
    let cargo_file = root.join("src/bin/tool.rs");
    let universe = HashSet::from([
        root.join("package.json"),
        direct.clone(),
        cargo_file.clone(),
    ]);
    let mut bins = CargoBinIndex::default();
    bins.insert(None, "tool".to_string(), cargo_file.clone());
    let mut resolver = WorkflowRunResolver::new(&root, &universe, &bins);

    assert_eq!(
        resolver.resolve("npm run cycle-a", &root),
        vec![root.join("package.json")]
    );
    assert_eq!(
        resolver.resolve("cargo run --bin tool", &root),
        vec![cargo_file]
    );
    assert_eq!(
        resolver.resolve("node scripts/direct.mjs", &root),
        vec![direct.clone()]
    );
    assert!(resolver.resolve("node ${SCRIPT}", &root).is_empty());
    assert!(resolver
        .resolve("cargo run --bin tool | tee output; echo ok", &root)
        .is_empty());
    assert!(resolver
        .resolve("npm run missing", &root.parent().unwrap().join("outside"))
        .is_empty());
    assert!(resolver.resolve("ONLY_ENV=set", &root).is_empty());
    let mut targets = HashSet::new();
    resolver.resolve_cargo_targets(&[], &mut targets);
    resolver.resolve_package_script("${DYNAMIC}", &root, &mut HashSet::new(), &mut targets);
    resolver.insert_local_path(
        direct.to_str().expect("UTF-8 fixture path"),
        &root,
        &mut targets,
    );
    assert!(targets.contains(&direct));
}

#[test]
fn workflow_working_directory_and_local_action_resolution_are_scoped() {
    let root = workflow_topology_fixture();
    let workflow: serde_yaml::Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: scripts\n").unwrap();
    let job: serde_yaml::Value =
        serde_yaml::from_str("defaults:\n  run:\n    working-directory: packages/tool\n").unwrap();
    let step: serde_yaml::Value =
        serde_yaml::from_str("working-directory: .\nrun: node scripts/direct.mjs\n").unwrap();
    let empty: serde_yaml::Value = serde_yaml::from_str("{}").unwrap();

    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &job, &step),
        Some(root.clone())
    );
    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &job, &empty),
        Some(root.join("packages/tool"))
    );
    assert_eq!(
        workflow_run_working_directory(&root, &workflow, &empty, &empty),
        Some(root.join("scripts"))
    );
    let dynamic: serde_yaml::Value =
        serde_yaml::from_str("working-directory: ${{ matrix.dir }}").unwrap();
    assert_eq!(
        workflow_run_working_directory(&root, &empty, &empty, &dynamic),
        None
    );

    let action = root.join(".github/actions/local/action.yml");
    let universe = HashSet::from([action.clone()]);
    let action_dirs = [root.join(".github/actions")];
    assert_eq!(
        resolve_local_action_descriptor(&root, "./.github/actions/local", &universe, &action_dirs,),
        Some(action)
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "actions/checkout@v4", &universe, &action_dirs),
        None
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "./${{ matrix.action }}", &universe, &action_dirs,),
        None
    );
    assert_eq!(
        resolve_local_action_descriptor(&root, "./../outside", &universe, &action_dirs),
        None
    );
    let outside_action = root.join("other-actions/local/action.yml");
    assert_eq!(
        resolve_local_action_descriptor(
            &root,
            "./other-actions/local",
            &HashSet::from([outside_action]),
            &action_dirs,
        ),
        None
    );
}
