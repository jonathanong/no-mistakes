use super::parse_program_with_resolver;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(in crate::integration_tests) fn parse_program(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    tsconfig: &TsConfig,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Vec<ConfigProject>> {
    let resolver = match visible_files {
        Some(visible) => ImportResolver::new(tsconfig).with_visible(visible),
        None => ImportResolver::new(tsconfig),
    };
    parse_program_with_resolver(program, source, path, config_dir, root, &resolver)
}
