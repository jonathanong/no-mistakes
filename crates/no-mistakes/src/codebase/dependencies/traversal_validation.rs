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
