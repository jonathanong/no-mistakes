use super::*;
use crate::codebase::ts_resolver::TsConfig;

pub(super) fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    super::prepare::resolve_tsconfig_from_visible(
        root,
        tsconfig,
        &crate::codebase::ts_source::discover_visible_paths(root),
    )
}

pub(super) fn detect_environment(path: &Path) -> Environment {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Environment::Unknown;
    };
    crate::ast::with_program(path, &source, |program, _| {
        let has_use_server = program
            .directives
            .iter()
            .any(|directive| directive.directive == "use server");
        let has_use_client = program
            .directives
            .iter()
            .any(|directive| directive.directive == "use client");
        match (has_use_server, has_use_client) {
            (true, _) => Environment::Server,
            (_, true) => Environment::Client,
            _ => Environment::Unknown,
        }
    })
    .unwrap_or(Environment::Unknown)
}
