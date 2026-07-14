#[derive(Deserialize, Default)]
struct PackageJson {
    name: Option<String>,
    workspaces: Option<WorkspacesField>,
    main: Option<String>,
    module: Option<String>,
    exports: Option<serde_json::Value>,
    imports: Option<serde_json::Value>,
    types: Option<String>,
    dependencies: Option<serde_json::Value>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<serde_json::Value>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<serde_json::Value>,
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<serde_json::Value>,
}

impl PackageJson {
    fn dependency_names(&self) -> std::collections::HashSet<String> {
        [
            &self.dependencies,
            &self.dev_dependencies,
            &self.peer_dependencies,
            &self.optional_dependencies,
        ]
        .into_iter()
        .filter_map(|field| field.as_ref()?.as_object())
        .flat_map(serde_json::Map::keys)
        .cloned()
        .collect()
    }
}

#[derive(Clone, Copy)]
enum WorkspaceSources<'a> {
    Filesystem,
    Store(&'a crate::codebase::ts_source::SourceStore),
}

impl WorkspaceSources<'_> {
    fn read(self, path: &Path) -> Result<std::sync::Arc<str>> {
        match self {
            Self::Filesystem => std::fs::read_to_string(path)
                .map(std::sync::Arc::<str>::from)
                .map_err(Into::into),
            Self::Store(store) => match store.read_path(path) {
                Ok(source) => Ok(source),
                Err(error) => Err(std::io::Error::new(error.kind(), error.to_string()).into()),
            },
        }
    }

    fn parse_json(self, path: &Path) -> Result<std::sync::Arc<serde_json::Value>> {
        match self {
            Self::Filesystem => {
                let source = std::fs::read_to_string(path)?;
                Ok(std::sync::Arc::new(serde_json::from_str(&source)?))
            }
            Self::Store(store) => match store.parse_json_path(path) {
                Ok(value) => Ok(value),
                Err(error) => Err(anyhow::Error::new(error).context(format!(
                    "failed to load workspace manifest {}",
                    path.display()
                ))),
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WorkspacesField {
    Array(Vec<String>),
    Object { packages: Vec<String> },
}

#[derive(Deserialize, Default)]
struct PnpmWorkspace {
    packages: Option<Vec<String>>,
}
