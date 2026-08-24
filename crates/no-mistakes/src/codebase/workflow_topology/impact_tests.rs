use super::yaml::normalize_entry;
use super::{topology_impact_report, CiTopologyImpactReport};
use std::path::{Path, PathBuf};

pub(super) struct Case {
    pub(super) name: &'static str,
    pub(super) roots: &'static [&'static str],
    pub(super) workflows: &'static [&'static str],
    pub(super) global: bool,
}

fn fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/workflow-topology-impact")
        .join(name);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().join("base");
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    replace_tree(&root, &fixture.path().join("head"));
    crate::test_support::git_commit_all(&root, "head");
    fixture
}

// The base/head trees are checked in. This harness only materializes those
// saved revisions into a temporary Git worktree, keeping the test cases
// reviewable without constructing YAML inline at runtime.
fn replace_tree(root: &Path, source: &Path) {
    let git_dir = root.join(".git");
    let mut paths = ignore::WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .map(|entry| entry.into_path())
        .filter(|path| path != root && !path.starts_with(&git_dir))
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in paths {
        if path.is_dir() {
            std::fs::remove_dir(path).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
    }
    copy_tree(source, root);
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in ignore::WalkBuilder::new(source)
        .hidden(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .filter(|entry| entry.path() != source)
    {
        let target = destination.join(entry.path().strip_prefix(source).unwrap());
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            std::fs::create_dir_all(target).unwrap();
        } else {
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

pub(super) fn report(name: &str) -> CiTopologyImpactReport {
    let fixture = fixture(name);
    topology_impact_report(&fixture.path().join("base"), "HEAD~", "HEAD", "ci.yml").unwrap()
}

pub(super) fn assert_case(case: Case) {
    let report = report(case.name);
    assert_eq!(report.global_fallback, case.global, "{}", case.name);
    assert_eq!(report.affected_root_job_ids, case.roots, "{}", case.name);
    assert_eq!(report.affected_workflows, case.workflows, "{}", case.name);
    assert!(report
        .changed_paths
        .windows(2)
        .all(|paths| paths[0] < paths[1]));
    assert!(report
        .affected_root_job_ids
        .windows(2)
        .all(|roots| roots[0] < roots[1]));
    assert!(report
        .affected_workflows
        .windows(2)
        .all(|workflows| workflows[0] < workflows[1]));
}

#[test]
fn revision_matrix_projects_only_reachable_owners() {
    for case in [
        Case {
            name: "unrelated-workflow",
            roots: &[],
            workflows: &[],
            global: false,
        },
        Case {
            name: "reusable-edit",
            roots: &[".github/workflows/ci.yml#test-tooling"],
            workflows: &[".github/workflows/ci.yml", ".github/workflows/tooling.yml"],
            global: false,
        },
        Case {
            name: "reusable-added",
            roots: &[".github/workflows/ci.yml#test-tooling"],
            workflows: &[".github/workflows/ci.yml", ".github/workflows/tooling.yml"],
            global: false,
        },
        Case {
            name: "reusable-deleted",
            roots: &[".github/workflows/ci.yml#test-tooling"],
            workflows: &[".github/workflows/ci.yml", ".github/workflows/tooling.yml"],
            global: false,
        },
        Case {
            name: "reusable-renamed",
            roots: &[".github/workflows/ci.yml#test-tooling"],
            workflows: &[
                ".github/workflows/ci.yml",
                ".github/workflows/tooling-next.yml",
                ".github/workflows/tooling.yml",
            ],
            global: false,
        },
    ] {
        assert_case(case);
    }
}

#[test]
fn entry_workflow_diffs_distinguish_job_local_from_global() {
    assert_case(Case {
        name: "entry-job-local",
        roots: &[".github/workflows/ci.yml#test-web"],
        workflows: &[".github/workflows/ci.yml"],
        global: false,
    });
    assert_case(Case {
        name: "entry-top-level",
        roots: &[
            ".github/workflows/ci.yml#test-backend",
            ".github/workflows/ci.yml#test-web",
        ],
        workflows: &[".github/workflows/ci.yml"],
        global: true,
    });
}

#[test]
fn topology_diagnostics_are_local_only_with_bound_root_evidence() {
    let localized = report("localized-diagnostic");
    assert!(!localized.global_fallback);
    let diagnostic = localized
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "workflow-call-cycle")
        .expect("localized workflow-cycle diagnostic");
    assert_eq!(format!("{:?}", diagnostic.scope), "Localized");
    assert_eq!(
        diagnostic.root_job_ids,
        Some(vec![".github/workflows/ci.yml#test-tooling".into()])
    );

    let global = report("parser-global");
    assert!(global.global_fallback);
    let diagnostic = global
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "malformed-workflow")
        .expect("global parser diagnostic");
    assert_eq!(format!("{:?}", diagnostic.scope), "Global");
    assert_eq!(diagnostic.root_job_ids, None);
}

#[test]
fn reports_are_deterministic_across_identical_revision_queries() {
    let fixture = fixture("reusable-renamed");
    let root = fixture.path().join("base");
    let first = topology_impact_report(&root, "HEAD~", "HEAD", "ci.yml").unwrap();
    let second = topology_impact_report(&root, "HEAD~", "HEAD", "ci.yml").unwrap();
    assert_eq!(first, second);
    assert_eq!(normalize_entry("ci.yml"), ".github/workflows/ci.yml");
}
