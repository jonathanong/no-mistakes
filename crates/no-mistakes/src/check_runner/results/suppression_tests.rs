use super::suppression::suppress_react;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::react_traits::Violation;
use std::path::PathBuf;

#[test]
fn line_less_react_findings_are_not_dropped_by_suppression_adapter() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-multiple");
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let mut findings = vec![Violation {
        component: "Fetcher".to_string(),
        file: "app/Fetcher.tsx".to_string(),
        rule: "assert-no-fetch".to_string(),
        detail: None,
    }];
    let mut suppressed = Vec::new();
    suppress_react(&root, &sources, &mut findings, &[], &mut suppressed);
    assert_eq!(findings.len(), 1);
    assert!(suppressed.is_empty());
}

#[test]
fn aggregate_react_suppression_uses_sidecar_locations_for_public_four_field_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-react-multiple");
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let mut findings = vec![Violation {
        component: "Fetcher".to_string(),
        file: "app/Fetcher.tsx".to_string(),
        rule: "assert-no-fetch".to_string(),
        detail: None,
    }];
    let targets = vec![vec![no_mistakes::react_traits::ReactSuppressionTarget {
        file: "app/Fetcher.tsx".to_string(),
        line: 5,
    }]];
    let mut suppressed = Vec::new();
    suppress_react(&root, &sources, &mut findings, &targets, &mut suppressed);
    assert!(findings.is_empty());
    assert_eq!(suppressed.len(), 1);
    assert_eq!(suppressed[0].directive.line, 4);
}
