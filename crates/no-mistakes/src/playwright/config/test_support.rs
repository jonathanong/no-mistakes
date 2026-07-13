use super::*;

pub(crate) fn load_settings(
    root: &Path,
    cli_config: Option<&Path>,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
) -> Result<Settings> {
    let visible_paths = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    super::load::load_settings_from_visible(
        root,
        cli_config,
        cli_playwright_configs,
        cli_project,
        &visible_paths,
    )
}
