use super::yaml::normalize_entry;
use super::{topology_impact_report, CiTopologyImpactReport};

pub(super) struct Case {
    pub(super) name: &'static str,
    pub(super) roots: &'static [&'static str],
    pub(super) workflows: &'static [&'static str],
    pub(super) global: bool,
}

fn fixture(name: &str) -> tempfile::TempDir {
    crate::test_support::materialize_workflow_topology_impact_fixture(name)
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
