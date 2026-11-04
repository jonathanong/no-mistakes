use super::{Options, RuleFinding};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn scan_with_config(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    files: &[PathBuf],
) -> Vec<RuleFinding> {
    super::scan_with_config_file(
        root,
        opts,
        files,
        crate::codebase::rules::configured_root::config_file_for_finding(root, config),
    )
}
