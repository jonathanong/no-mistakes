use super::*;

#[test]
fn signature_impact_traverses_type_only_workspace_edges() {
    assert!(signature_impact_edges().contains(&EdgeKind::WorkspaceTypeImport));
}
