use super::*;
use crate::codebase::ts_resolver::{find_tsconfig_from_visible, load_tsconfig, TsConfig};

pub(super) fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    match tsconfig {
        // Resolve a relative explicit tsconfig against `root`, not the cwd.
        Some(path) if path.is_absolute() => load_tsconfig(path),
        Some(path) => load_tsconfig(&root.join(path)),
        None => match find_tsconfig_from_visible(root, &visible_paths) {
            Some(path) => match load_tsconfig(&path) {
                Ok(config) => Ok(config),
                Err(_) => Ok(TsConfig {
                    dir: root.to_path_buf(),
                    paths: vec![],
                    paths_dir: root.to_path_buf(),
                    base_url: None,
                }),
            },
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
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
