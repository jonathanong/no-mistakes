fn add_ci_edges(root: &Path, all_files: &[PathBuf], forward: &mut EdgeMap, reverse: &mut EdgeMap) {
    let bins = collect_cargo_bins(root, all_files);
    if bins.is_empty() {
        return;
    }

    // Walk .github/workflows/*.yml
    let workflows_dir = root.join(".github").join("workflows");
    if !workflows_dir.is_dir() {
        return;
    }

    let edges: Vec<Edge> = all_files
        .par_iter()
        .filter(|path| path.starts_with(&workflows_dir))
        .filter(|path| {
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("yml" | "yaml")
            )
        })
        .flat_map_iter(|path| {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            let Ok(invocations) = crate::codebase::ci_workflows::extract_invocations(&source)
            else {
                return Vec::new();
            };

            let mut edges = Vec::new();
            for inv in invocations {
                let cargo_target_files = inv
                    .cargo_targets
                    .iter()
                    .filter_map(|target| bins.get_cargo_target(target));
                let direct_binary_files = inv
                    .binaries
                    .iter()
                    .filter(|binary_name| {
                        !inv.cargo_targets
                            .iter()
                            .any(|target| target.binary == **binary_name)
                    })
                    .filter_map(|binary_name| bins.by_name.get(binary_name));
                for source_file in cargo_target_files.chain(direct_binary_files) {
                    edges.push((
                        NodeId::File(path.clone()),
                        NodeId::File(source_file.clone()),
                        EdgeKind::CiInvocation,
                    ));
                }
            }
            edges
        })
        .collect();
    merge_edges(forward, reverse, edges);
}

#[derive(Default)]
struct CargoBinIndex {
    by_name: HashMap<String, PathBuf>,
    by_package_and_name: HashMap<(String, String), PathBuf>,
}

impl CargoBinIndex {
    fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    fn insert(&mut self, package: Option<&str>, name: String, source_file: PathBuf) {
        self.by_name
            .entry(name.clone())
            .or_insert_with(|| source_file.clone());
        if let Some(package) = package {
            self.by_package_and_name
                .insert((package.to_string(), name), source_file);
        }
    }

    fn get_cargo_target(
        &self,
        target: &crate::codebase::ci_workflows::CargoTarget,
    ) -> Option<&PathBuf> {
        target
            .package
            .as_ref()
            .and_then(|package| {
                self.by_package_and_name
                    .get(&(package.clone(), target.binary.clone()))
            })
            .or_else(|| self.by_name.get(&target.binary))
    }
}

fn collect_cargo_bins(root: &Path, all_files: &[PathBuf]) -> CargoBinIndex {
    let root_manifest = root.join("Cargo.toml");
    let root_toml = match std::fs::read_to_string(&root_manifest) {
        Ok(s) => s,
        Err(_) => return CargoBinIndex::default(),
    };

    let mut bins = CargoBinIndex::default();
    add_manifest_bins(&root_manifest, &root_toml, &mut bins);

    let members = match crate::codebase::ci_workflows::parse_cargo_workspace_members(&root_toml) {
        Ok(members) => members,
        Err(_) => return bins,
    };
    let excludes = crate::codebase::ci_workflows::parse_cargo_workspace_excludes(&root_toml)
        .unwrap_or_default();
    let member_set = cargo_member_globset(&members);
    let exclude_set = cargo_member_globset(&excludes);

    for (manifest, parent) in all_files
        .iter()
        .filter(|path| {
            path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml")
                && path != &&root_manifest
        })
        .filter_map(|manifest| manifest.parent().map(|parent| (manifest, parent)))
    {
        let Ok(rel_dir) = parent.strip_prefix(root) else {
            continue;
        };
        let is_member = member_set
            .as_ref()
            .map(|set| set.is_match(rel_dir))
            .unwrap_or(true);
        if !is_member
            || exclude_set
                .as_ref()
                .is_some_and(|set| set.is_match(rel_dir))
        {
            continue;
        }
        let Ok(cargo_toml) = std::fs::read_to_string(manifest) else {
            continue;
        };
        add_manifest_bins(manifest, &cargo_toml, &mut bins);
    }

    bins
}

fn cargo_member_globset(members: &[String]) -> Option<globset::GlobSet> {
    if members.is_empty() {
        return None;
    }
    let mut builder = globset::GlobSetBuilder::new();
    for member in members {
        let glob = globset::GlobBuilder::new(member)
            .literal_separator(true)
            .build()
            .ok()?;
        builder.add(glob);
    }
    builder.build().ok()
}

fn add_manifest_bins(manifest: &Path, cargo_toml: &str, bins: &mut CargoBinIndex) {
    let Ok(parsed_bins) = crate::codebase::ci_workflows::parse_cargo_bins(cargo_toml) else {
        return;
    };
    let package = crate::codebase::ci_workflows::parse_cargo_package_name(cargo_toml)
        .ok()
        .flatten();
    let Some(manifest_dir) = manifest.parent() else {
        return;
    };
    for (name, rel_path) in parsed_bins {
        if let Some(source_file) = resolve_cargo_bin_source(manifest_dir, &name, &rel_path) {
            bins.insert(package.as_deref(), name, source_file);
        }
    }
}

fn resolve_cargo_bin_source(manifest_dir: &Path, name: &str, rel_path: &str) -> Option<PathBuf> {
    let declared = manifest_dir.join(rel_path);
    if declared.exists() {
        return Some(declared);
    }

    let nested = manifest_dir
        .join("src")
        .join("bin")
        .join(name)
        .join("main.rs");
    if nested.exists() {
        return Some(nested);
    }

    None
}
