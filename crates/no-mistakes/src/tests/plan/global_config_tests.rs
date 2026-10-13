use super::*;

#[test]
fn pnpm_workspace_configuration_is_a_root_scoped_broad_trigger() {
    let root = Path::new("/repo");
    assert!(is_global_config_path(
        root,
        &root.join("pnpm-workspace.yaml"),
        "pnpm-workspace.yaml"
    ));
    assert!(!is_global_config_path(
        root,
        &root.join("packages/app/pnpm-workspace.yaml"),
        "packages/app/pnpm-workspace.yaml"
    ));
}
