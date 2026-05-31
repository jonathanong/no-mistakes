fn resolve_root(arg: Option<&Path>, cwd: &Path) -> PathBuf {
    match arg {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd.to_path_buf(),
    }
}

pub(crate) fn resolve_frontend_root(
    arg: Option<&Path>,
    root: &Path,
    config: Option<&Config>,
) -> Result<PathBuf> {
    if let Some(path) = arg {
        let frontend_root = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        return validate_frontend_root(frontend_root);
    }

    let Some(config) = config else {
        return default_frontend_root(root);
    };
    let opts: RouteOptions = config.rule_options("route-consistency");
    if opts == RouteOptions::default() || opts.frontend_root.is_empty() {
        return default_frontend_root(root);
    }

    validate_frontend_root(root.join(opts.frontend_root))
}

fn default_frontend_root(root: &Path) -> Result<PathBuf> {
    let default = root.join(DEFAULT_FRONTEND_ROOT);
    if default.is_dir() {
        return Ok(default);
    }

    bail!(
        "could not determine Next.js App Router root; pass --frontend-root or configure route-consistency.frontendRoot"
    )
}

fn validate_frontend_root(frontend_root: PathBuf) -> Result<PathBuf> {
    if frontend_root.is_dir() {
        Ok(frontend_root)
    } else {
        bail!(
            "Next.js App Router root does not exist: {}",
            frontend_root.display()
        )
    }
}

fn test_globs_or_default(root: &Path, globs: &[String]) -> Vec<String> {
    if globs.is_empty() {
        crate::config::v2::load_v2_config(root, None)
            .ok()
            .and_then(|config| {
                crate::codebase::test_discovery::discover_tests(
                    root,
                    &config,
                    crate::codebase::test_discovery::TestRunner::Playwright,
                )
                .ok()
            })
            .map(|discovered| {
                discovered
                    .tests
                    .iter()
                    .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
                    .collect()
            })
            .filter(|globs: &Vec<String>| !globs.is_empty())
            .unwrap_or_else(|| crate::codebase::dependencies::test_globs("playwright"))
    } else {
        globs.to_vec()
    }
}
