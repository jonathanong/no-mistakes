use super::*;
use crate::tests::{Confidence, ImpactReason, ResourceCallSite, SelectedTest, Warning};

#[test]
fn plan_backed_why_preserves_resource_edge_detail() {
    let plan = TestPlan {
        changed_files: Vec::new(),
        selected_tests: vec![SelectedTest {
            test_file: "src/load.test.ts".to_string(),
            confidence: Confidence::High,
            targets: Vec::new(),
            reasons: vec![ImpactReason {
                changed_file: "resources/schema.sql".to_string(),
                path: vec![
                    "resources/schema.sql".to_string(),
                    "src/load.ts".to_string(),
                    "src/load.test.ts".to_string(),
                ],
                via: vec!["resource".to_string(), "dependency".to_string()],
                via_details: vec![
                    Some(ImpactEdgeDetail::Resource {
                        consumer_file: "src/load.ts".to_string(),
                        call_sites: vec![ResourceCallSite {
                            call_kind: "read-file-sync".to_string(),
                            line: 5,
                        }],
                    }),
                    None,
                ],
            }],
        }],
        groups: Vec::new(),
        warnings: Vec::<Warning>::new(),
        fallback_triggered: false,
        fallback_reason: None,
        ..Default::default()
    };
    let result = steps_from_plan(&plan, "src/load.test.ts", None);
    let steps = result.get("resources/schema.sql").unwrap();
    assert!(matches!(
        steps[0].detail,
        Some(ImpactEdgeDetail::Resource { ref call_sites, .. })
            if call_sites == &[ResourceCallSite {
                call_kind: "read-file-sync".to_string(),
                line: 5,
            }]
    ));
    assert_eq!(steps[1].detail, None);
}
