fn add_ci_edges(
    root: &Path,
    all_files: &[PathBuf],
    parsed: &crate::codebase::ci_workflows::ParsedWorkflowSet,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
    interner: &PathInterner,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) {
    let bins = collect_cargo_bins(root, all_files, sources);
    if bins.is_empty() {
        return;
    }

    let edges: Vec<Edge> = parsed
        .documents
        .par_iter()
        .flat_map_iter(|document| {
            let Ok(value) = document.value.as_ref() else {
                return Vec::new();
            };
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(&document.path));
            let invocations = crate::codebase::ci_workflows::extract_invocations_value(value);

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
                        NodeId::file_in(interner, path.clone()),
                        NodeId::file_in(interner, source_file.clone()),
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
        match &target.package {
            Some(package) => self
                .by_package_and_name
                .get(&(package.clone(), target.binary.clone())),
            None => self.by_name.get(&target.binary),
        }
    }
}

fn collect_cargo_bins(
    root: &Path,
    all_files: &[PathBuf],
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> CargoBinIndex {
    let visible_files: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let root_manifest = crate::codebase::ts_resolver::normalize_path(&root.join("Cargo.toml"));
    if !visible_files.contains(&root_manifest) {
        return CargoBinIndex::default();
    }
    let Some(root_toml) =
        crate::codebase::ts_source::SourceStore::read_optional(sources, &root_manifest)
    else {
        return CargoBinIndex::default();
    };

    let mut bins = CargoBinIndex::default();
    add_manifest_bins(&root_manifest, &root_toml, &visible_files, &mut bins);

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
        let Some(cargo_toml) =
            crate::codebase::ts_source::SourceStore::read_optional(sources, manifest)
        else {
            continue;
        };
        add_manifest_bins(manifest, &cargo_toml, &visible_files, &mut bins);
    }

    bins
}

include!("edge_ci_manifest.rs");
