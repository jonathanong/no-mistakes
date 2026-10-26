use super::{parse_program_with_resolver, ParsedPlaywrightConfig};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::Path;

pub(in crate::integration_tests) fn parse_program(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
    tsconfig: &TsConfig,
    visible_files: Option<&crate::fx::PathSet>,
) -> Result<ParsedPlaywrightConfig> {
    let resolver = match visible_files {
        Some(visible) => ImportResolver::new(tsconfig).with_visible(visible),
        None => ImportResolver::new(tsconfig),
    };
    parse_program_with_resolver(program, source, path, config_dir, &resolver)
}

impl ParsedPlaywrightConfig {
    pub(in crate::integration_tests) fn into_projects(
        self,
        root: &Path,
        config: &str,
    ) -> Vec<crate::integration_tests::types::ConfigProject> {
        self.into_projects_from_visible(root, config, None)
            .expect("regex projection requires the caller-owned visible inventory")
    }
}
