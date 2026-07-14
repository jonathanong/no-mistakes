use super::super::{
    discover_workflow_files, discover_workflow_files_from_snapshot, relative_slash, WorkflowSet,
};
use crate::config::v2::schema::CiConfig;

#[test]
fn configured_workflow_dir_uses_nested_git_visibility() {
    let dir = crate::test_support::materialize_gitignore_fixture("ci-nested-worktree");
    let nested = dir.path().join("nested-worktree");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_init(&nested);
    std::fs::copy(
        nested.join("git-info-exclude.fixture"),
        nested.join(".git/info/exclude"),
    )
    .unwrap();
    crate::test_support::git_add_all(&nested);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(dir.path());

    let ci = CiConfig {
        workflow_dirs: vec!["nested-worktree/.github/workflows".to_string()],
        ..CiConfig::default()
    };
    let files: Vec<String> = discover_workflow_files(dir.path(), &ci)
        .iter()
        .map(|path| relative_slash(dir.path(), path))
        .collect();

    assert_eq!(files, vec!["nested-worktree/.github/workflows/visible.yml"]);
    assert_eq!(
        discover_workflow_files_from_snapshot(dir.path(), &ci, &snapshot),
        vec![nested.join(".github/workflows/visible.yml")]
    );
    assert_eq!(WorkflowSet::load(dir.path(), &ci).workflows.len(), 1);
    let public = WorkflowSet::load(dir.path(), &ci);
    let prepared = WorkflowSet::load_from_snapshot(dir.path(), &ci, &snapshot);
    assert_eq!(prepared.workflows, public.workflows);
    assert_eq!(prepared.warnings, public.warnings);
}

#[test]
fn configured_workflow_dir_outside_root_uses_its_own_git_visibility() {
    let dir = crate::test_support::materialize_gitignore_fixture("ci-outside-root");
    let root = dir.path().join("project-root");
    let workflows = dir.path().join("external-workflows");
    crate::test_support::git_init(&root);
    crate::test_support::git_init(&workflows);
    std::fs::copy(
        workflows.join("git-info-exclude.fixture"),
        workflows.join(".git/info/exclude"),
    )
    .unwrap();
    crate::test_support::git_add_all(&workflows);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);

    let ci = CiConfig {
        workflow_dirs: vec!["../external-workflows".to_string()],
        ..CiConfig::default()
    };
    let files = discover_workflow_files(&root, &ci);

    assert_eq!(files, vec![workflows.join("visible.yaml")]);
    assert_eq!(
        discover_workflow_files_from_snapshot(&root, &ci, &snapshot),
        vec![workflows.join("visible.yaml")]
    );
    assert_eq!(WorkflowSet::load(&root, &ci).workflows.len(), 1);
}
