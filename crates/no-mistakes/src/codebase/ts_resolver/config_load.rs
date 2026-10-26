fn load_tsconfig_inner(
    path: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    locate_paths: bool,
) -> Result<TsConfigFound> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;

    if !visited.insert(canonical.clone()) {
        anyhow::bail!("tsconfig.extends cycle detected at {}", path.display());
    }

    let dir = path
        .parent()
        .context(format!("resolving parent directory for {}", path.display()))?
        .to_path_buf();

    let content = match sources {
        Some(sources) => sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))?,
        None => std::sync::Arc::<str>::from(
            std::fs::read_to_string(path).context(format!("reading {}", path.display()))?,
        ),
    };

    let parsed = jsonc_parser::parse_to_ast(
        &content,
        &jsonc_parser::CollectOptions::default(),
        &jsonc_parser::ParseOptions::default(),
    )
    .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
    let own_paths = parsed
        .value
        .as_ref()
        .and_then(jsonc_parser::ast::Value::as_object)
        .and_then(|root| last_object_property(root, "compilerOptions"))
        .and_then(|compiler_options| last_object_property(compiler_options, "paths"))
        .map(|paths| collect_path_mappings(paths, &content, path, locate_paths));
    let v = parsed
        .value
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null);
    let own_base_url = v
        .get("compilerOptions")
        .and_then(|co| co.get("baseUrl"))
        .and_then(|p| p.as_str())
        .map(|base_url| normalize_path(&dir.join(base_url)));

    // Check for an extends chain.
    // `extends` may be a string (pre-TS 5.0) or an array of strings (TS 5.0+).
    // For arrays, TS 5.0 applies them left-to-right; the rightmost definition wins
    // for any given property. `paths` and `baseUrl` inherit independently, so a
    // later config that only defines `baseUrl` must not discard earlier `paths`.
    let extends_list: Vec<&str> = match v.get("extends") {
        None => vec![],
        Some(serde_json::Value::String(s)) => vec![s.as_str()],
        Some(serde_json::Value::Array(arr)) => {
            let mut list = Vec::with_capacity(arr.len());
            for item in arr {
                let s = item.as_str().ok_or_else(|| {
                    anyhow::anyhow!(
                        "{} extends array contains a non-string entry; \
                         TypeScript rejects non-string extends values",
                        path.display()
                    )
                })?;
                list.push(s);
            }
            list
        }
        Some(other) => anyhow::bail!(
            "{} extends must be a string or array of strings, got: {}; \
             TypeScript rejects this configuration",
            path.display(),
            other
        ),
    };

    let mut inherited_paths: Option<TsConfigFound> = None;
    let mut inherited_base_url: Option<PathBuf> = None;
    if !extends_list.is_empty() {
        for extends_raw in &extends_list {
            // Bare specifiers (npm packages like "@scope/tsconfig") cannot be resolved
            // without a node_modules lookup. Emit a warning and skip — the caller may
            // have local paths defined, or another entry in the extends array may provide
            // them. If alias resolution is unexpectedly empty, this warning identifies why.
            if !extends_raw.starts_with('.') && !std::path::Path::new(extends_raw).is_absolute() {
                eprintln!(
                    "warning: {} extends npm package '{}'; \
                     path aliases from that package cannot be resolved without node_modules",
                    path.display(),
                    extends_raw
                );
                continue;
            }
            let base_path = normalize_path(&dir.join(extends_raw));
            let base_path = if base_path.is_dir() {
                base_path.join("tsconfig.json")
            } else if base_path.extension().is_none() {
                base_path.with_extension("json")
            } else {
                base_path
            };
            let base = load_tsconfig_inner(&base_path, visited, sources, locate_paths)
                .context(format!("loading extended tsconfig {}", base_path.display()))?;
            if let Some(base_url) = &base.inner.base_url {
                inherited_base_url = Some(base_url.clone());
            }
            if base.paths_found {
                inherited_paths = Some(base);
            }
        }
    }

    let base_url = own_base_url.or(inherited_base_url);

    // Child defines its own paths (even if empty). An explicit `paths: {}`
    // overrides any paths from the extends chain, but not an inherited baseUrl.
    if let Some((paths, locations)) = own_paths {
        let paths_dir = dir.clone();
        visited.remove(&canonical);
        return Ok(TsConfigFound {
            inner: TsConfig {
                dir,
                paths,
                paths_dir,
                base_url,
            },
            paths_found: true,
            locations,
        });
    }

    if let Some(base) = inherited_paths {
        visited.remove(&canonical);
        return Ok(TsConfigFound {
            inner: TsConfig {
                dir,
                paths: base.inner.paths,
                paths_dir: base.inner.paths_dir,
                base_url,
            },
            paths_found: true,
            locations: base.locations,
        });
    }

    visited.remove(&canonical);
    Ok(TsConfigFound {
        inner: TsConfig {
            dir: dir.clone(),
            paths: vec![],
            paths_dir: dir,
            base_url,
        },
        paths_found: false,
        locations: None,
    })
}
