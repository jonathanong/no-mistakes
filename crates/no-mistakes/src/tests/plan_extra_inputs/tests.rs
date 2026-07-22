use super::*;
use no_mistakes::codebase::dependencies::graph::VitestSetupField;

#[test]
fn deleted_transitive_details_align_with_synthetic_and_vitest_edges() {
    let edge_path = [
        EdgeKind::Import,
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
    ];
    let via = std::iter::once("deleted-dependency")
        .chain(edge_path.iter().map(|kind| impact_reason_label(*kind)))
        .collect::<Vec<_>>();
    let via_details = deleted_transitive_via_details(&edge_path);

    assert_eq!(via.len(), via_details.len());
    assert_eq!(via_details[0], None, "synthetic edge has no detail");
    assert_eq!(via_details[1], None, "import has no detail");
    assert_eq!(
        via_details[2],
        Some(ImpactEdgeDetail::VitestSetup {
            field: "setupFiles".to_string(),
        })
    );
}
