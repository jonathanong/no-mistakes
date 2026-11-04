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

#[test]
fn fragments_and_all_automatic_extensions_are_global_configuration_inputs() {
    let root = Path::new("/repo");
    for relative in [
        ".no-mistakes.json",
        ".no-mistakes.jsonc",
        ".no-mistakes/tests/nested/plan.yml",
    ] {
        assert!(is_global_config_path(root, &root.join(relative), relative));
    }
    assert!(!is_global_config_path(
        root,
        &root.join("package/.no-mistakes/rules/a.yml"),
        "package/.no-mistakes/rules/a.yml"
    ));
}
