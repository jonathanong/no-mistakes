// Local `include` entries from the root `pnpm-workspace.yaml` (pnpm 12.15).
// Config-dependency sources (not `./` or `../`) are ignored. Included files
// are not scanned for their own `include` key. A list grant reserves a
// setting even when the file omits it; `*` owns only settings that file
// defines. Two claimants is a conflict — callers must not pick a winner.

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocKind {
    Root,
    Star,
    Listed,
}

struct Doc {
    label: String,
    path: PathBuf,
    base_dir: PathBuf,
    dir_rel: String,
    body: serde_yaml::Mapping,
    kind: DocKind,
    listed: Vec<String>,
}

struct Phantom {
    label: String,
    kind: DocKind,
    listed: Vec<String>,
}

pub(crate) enum PackageGlobs {
    DefaultStar,
    List(Vec<String>),
    Conflict,
}

pub(crate) enum ReleaseAgeExclude {
    FromRoot,
    Included(Vec<serde_yaml::Value>),
    Conflict,
}

pub(crate) struct IncludeLoadError {
    pub message: String,
    pub settings: Vec<String>,
}

impl IncludeLoadError {
    pub(crate) fn affects(&self, setting: &str) -> bool {
        self.settings
            .iter()
            .any(|item| item == setting || item == "*")
    }
}

pub(crate) struct CatalogHit {
    pub specifier: String,
    pub base_dir: PathBuf,
}

#[derive(Default)]
pub(crate) struct CatalogIndex {
    pub(crate) default_entries: std::collections::BTreeMap<String, CatalogHit>,
    pub(crate) named:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, CatalogHit>>,
}

impl CatalogIndex {
    pub(crate) fn get(&self, catalog: &str, package: &str) -> Option<&CatalogHit> {
        if catalog.is_empty() || catalog == "default" {
            self.default_entries.get(package)
        } else {
            self.named.get(catalog)?.get(package)
        }
    }
}

pub(crate) struct PnpmInclude {
    pub packages: PackageGlobs,
    /// The root file's own `packages` list. A conflict still reports both
    /// claimants; this list is not a chosen winner.
    pub root_packages: Vec<String>,
    pub packages_invalid: bool,
    pub duplicates: Vec<String>,
    pub included_override_paths: Vec<PathBuf>,
    pub release_age_exclude: ReleaseAgeExclude,
    pub catalogs: CatalogIndex,
    pub load_errors: Vec<IncludeLoadError>,
}

pub(crate) fn resolve_pnpm_include_at_root(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Option<PnpmInclude>> {
    let path = normalize_path(&root.join("pnpm-workspace.yaml"));
    let content = match sources.read_path(&path) {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };
    resolve_pnpm_include(root, &content, |path| sources.read_path(path).ok()).map(Some)
}

pub(crate) fn resolve_pnpm_include(
    root: &Path,
    yaml: &str,
    mut read: impl FnMut(&Path) -> Option<std::sync::Arc<str>>,
) -> Result<PnpmInclude> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    let Some(root_map) = value.as_mapping().cloned() else {
        return Err(anyhow::Error::msg("pnpm-workspace.yaml must be a mapping"));
    };
    let root_norm = normalize_path(root);
    let mut docs = vec![Doc {
        label: "pnpm-workspace.yaml".to_string(),
        path: root_norm.join("pnpm-workspace.yaml"),
        base_dir: root_norm.clone(),
        dir_rel: String::new(),
        body: root_map.clone(),
        kind: DocKind::Root,
        listed: Vec::new(),
    }];
    let mut phantoms = Vec::new();
    let mut load_errors = Vec::new();
    load_includes(
        &root_norm,
        &root_map,
        &mut read,
        &mut docs,
        &mut phantoms,
        &mut load_errors,
    );
    let mut duplicates = Vec::new();
    let packages_invalid = packages_field_invalid(&root_map);
    let root_packages = root_package_list(&root_map);
    let packages = package_globs(&docs, &phantoms, &mut duplicates);
    let included_override_paths = override_paths(&docs, &phantoms, &mut duplicates);
    let release_age_exclude = release_age(&docs, &phantoms, &mut duplicates);
    let catalogs = fill_catalogs(&docs, &phantoms, &mut duplicates);
    duplicates.sort();
    duplicates.dedup();
    Ok(PnpmInclude {
        packages,
        root_packages,
        packages_invalid,
        duplicates,
        included_override_paths,
        release_age_exclude,
        catalogs,
        load_errors,
    })
}
