#[derive(Clone)]
enum Cached {
    Ready(classify::Classified),
    Failed(String),
}

impl Cached {
    fn into_result(self) -> Result<classify::Classified> {
        match self {
            Self::Ready(classified) => Ok(classified),
            Self::Failed(message) => anyhow::bail!("{message}"),
        }
    }
}

fn remember(
    cache: &RefCell<HashMap<PathBuf, Cached>>,
    file: &Path,
    load: impl FnOnce() -> Result<classify::Classified>,
) -> Result<classify::Classified> {
    if let Some(entry) = cache.borrow().get(file) {
        return entry.clone().into_result();
    }
    let entry = match load() {
        Ok(classified) => Cached::Ready(classified),
        Err(error) => Cached::Failed(error.to_string()),
    };
    cache.borrow_mut().insert(file.to_path_buf(), entry.clone());
    entry.into_result()
}

fn ready(cache: &RefCell<HashMap<PathBuf, Cached>>, file: &Path) -> Result<classify::Classified> {
    cache
        .borrow()
        .get(file)
        .cloned()
        .map(Cached::into_result)
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("missing setup classification for {}", file.display()))
}

fn load_setup(group: &Group<'_>, file: &Path) -> Result<classify::Classified> {
    let Some(facts) = group.shared.ts.get(file) else {
        anyhow::bail!("missing shared facts for {}", file.display());
    };
    if let Some(error) = &facts.parse_error {
        anyhow::bail!("failed to parse {}: {error}", file.display());
    }
    let Some(dynamic) = facts.dynamic_imports.as_ref() else {
        anyhow::bail!("missing dynamic import facts for {}", file.display());
    };
    Ok(dynamic.setup_reachability.clone().into())
}

fn load_helper(group: &Group<'_>, file: &Path) -> Result<classify::Classified> {
    let Some(facts) = group.shared.ts.get(file) else {
        anyhow::bail!(
            "missing shared facts for imported setup helper {}",
            file.display()
        );
    };
    if let Some(error) = &facts.parse_error {
        anyhow::bail!(
            "failed to parse imported setup helper {}: {error}",
            file.display()
        );
    }
    let Some(dynamic) = facts.dynamic_imports.as_ref() else {
        anyhow::bail!(
            "missing dynamic import facts for imported setup helper {}",
            file.display()
        );
    };
    Ok(dynamic.setup_reachability.clone().into())
}
