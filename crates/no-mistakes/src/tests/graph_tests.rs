use super::*;
use crate::tests::{Confidence, ImpactReason, ResourceCallSite, SelectedTest, TestPlan, Warning};

fn resource_detail(line: u32) -> ImpactEdgeDetail {
    ImpactEdgeDetail::Resource {
        consumer_file: "src/load-schema.ts".to_string(),
        call_sites: vec![ResourceCallSite {
            call_kind: "read-file-sync".to_string(),
            line,
        }],
    }
}

#[test]
fn json_graph_keeps_and_merges_resource_provenance() {
    let reason = |line| ImpactReason {
        changed_file: "db/schema.sql".to_string(),
        path: vec![
            "db/schema.sql".to_string(),
            "src/load-schema.ts".to_string(),
            "tests/schema.test.ts".to_string(),
        ],
        via: vec!["resource".to_string(), "dependency".to_string()],
        via_details: vec![Some(resource_detail(line)), None],
    };
    let plan = TestPlan {
        changed_files: Vec::new(),
        selected_tests: vec![SelectedTest {
            test_file: "tests/schema.test.ts".to_string(),
            confidence: Confidence::High,
            reasons: vec![reason(3), reason(7)],
            targets: Vec::new(),
        }],
        groups: Vec::new(),
        warnings: Vec::<Warning>::new(),
        fallback_triggered: false,
        fallback_reason: None,
        ..Default::default()
    };

    let graph = graph_json(&plan).unwrap();
    let resource = graph
        .edges
        .iter()
        .find(|edge| edge.via == "resource")
        .unwrap();
    let Some(ImpactEdgeDetail::Resource { call_sites, .. }) = &resource.detail else {
        panic!("resource detail must be retained");
    };
    assert_eq!(
        call_sites.iter().map(|site| site.line).collect::<Vec<_>>(),
        [3, 7]
    );
    assert!(
        !serde_json::to_string(&graph)
            .unwrap()
            .contains("\"detail\":null")
    );
}

#[test]
fn legacy_reason_omits_empty_or_none_details() {
    let reason = ImpactReason {
        changed_file: "src/a.ts".to_string(),
        path: vec!["src/a.ts".to_string(), "tests/a.test.ts".to_string()],
        via: vec!["dependency".to_string()],
        via_details: vec![None],
    };
    let value = serde_json::to_value(reason).unwrap();
    assert!(value.get("via_details").is_none());
}

#[test]
fn resource_call_sites_merge_by_line_then_kind() {
    let mut sites = vec![
        ResourceCallSite {
            call_kind: "read-file-sync".to_string(),
            line: 9,
        },
        ResourceCallSite {
            call_kind: "glob".to_string(),
            line: 3,
        },
    ];
    merge_call_sites(
        &mut sites,
        &[
            ResourceCallSite {
                call_kind: "read-file".to_string(),
                line: 3,
            },
            ResourceCallSite {
                call_kind: "glob".to_string(),
                line: 3,
            },
        ],
    );
    assert_eq!(
        sites
            .iter()
            .map(|site| (site.line, site.call_kind.as_str()))
            .collect::<Vec<_>>(),
        [(3, "glob"), (3, "read-file"), (9, "read-file-sync")]
    );
}
