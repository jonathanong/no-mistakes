use crate::codebase::ts_source::is_portably_absolute_path;
use crate::config::source_locations::ConfigPath;
use crate::config::v2::schema::{invalid_option, RuleDef};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Component, Path};

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ShellcheckOptions {
    pub(super) severity: Severity,
}

#[derive(Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(super) enum Severity {
    Error,
    #[default]
    Warning,
    Info,
    Style,
}

impl Severity {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Style => "style",
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) shell_files: Vec<String>,
    pub(crate) shebang_dirs: Vec<String>,
    pub(crate) tracked_only: bool,
    pub(super) shellcheck: ShellcheckOptions,
    pub(super) skills_lockfile: Option<String>,
}

impl Options {
    pub(in crate::codebase::rules) fn validate_paths(
        &self,
        rule: &RuleDef,
        root: &Path,
    ) -> Result<()> {
        validate_shell_files(rule, root, &self.shell_files)?;
        for (index, path) in self.shebang_dirs.iter().enumerate() {
            if !repository_relative(path) {
                return Err(path_error(
                    rule,
                    "shebangDirs",
                    Some(index),
                    path,
                    "a repository-relative directory path",
                ));
            }
        }
        if let Some(path) = &self.skills_lockfile {
            if path.trim().is_empty() || !repository_relative(path) {
                return Err(path_error(
                    rule,
                    "skillsLockfile",
                    None,
                    path,
                    "a repository-relative file path",
                ));
            }
        }
        Ok(())
    }
}

fn validate_shell_files(rule: &RuleDef, root: &Path, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let canonical_root = root.canonicalize()?;
    for (index, path) in paths.iter().enumerate() {
        if path.trim().is_empty()
            || !repository_relative(path)
            || !root
                .join(path)
                .canonicalize()
                .is_ok_and(|resolved| resolved.starts_with(&canonical_root) && resolved.is_file())
        {
            return Err(path_error(
                rule,
                "shellFiles",
                Some(index),
                path,
                "an existing repository-relative file path",
            ));
        }
    }
    Ok(())
}

fn repository_relative(raw: &str) -> bool {
    let path = Path::new(raw);
    !is_portably_absolute_path(path)
        && !matches!(path.components().next(), Some(Component::Prefix(_)))
        && !raw.contains('\0')
        && !raw.split(['/', '\\']).any(|part| part == "..")
}

fn path_error(
    rule: &RuleDef,
    key: &str,
    index: Option<usize>,
    value: &str,
    expected: &str,
) -> anyhow::Error {
    let path = ConfigPath::default().key("options").key(key);
    let path = match index {
        Some(index) => path.index(index),
        None => path,
    };
    invalid_option(
        rule,
        path,
        format_args!("expected {expected}, got {value:?}"),
    )
}
