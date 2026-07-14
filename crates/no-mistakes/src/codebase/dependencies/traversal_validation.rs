fn resolve_root(args: &TraverseArgs, cwd: &Path) -> PathBuf {
    match &args.root {
        Some(path) => {
            if path.is_absolute() {
                path.clone()
            } else {
                cwd.join(path)
            }
        }
        None => cwd.to_path_buf(),
    }
}

fn validate_direction(direction: &Direction, entrypoints: &[Entrypoint]) -> Result<()> {
    if matches!(direction, Direction::Deps) {
        for entrypoint in entrypoints {
            if entrypoint.symbol.is_some()
                && !matches!(entrypoint.node, NodeId::Symbol { .. })
            {
                bail!(
                    "#symbol targeting (e.g. `file.mts#exportName`) is only supported \
                     in the `dependents` direction unless --symbols is enabled."
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn traversal_needs_symbol_facts(args: &TraverseArgs) -> bool {
    args.include_symbols
        || args.file_symbols.iter().any(Option::is_some)
        || args.files.iter().enumerate().any(|(index, file)| {
            !args
                .file_entrypoints_are_structured
                .get(index)
                .copied()
                .unwrap_or(false)
                && parse_entrypoint(&file.to_string_lossy()).1.is_some()
        })
}
