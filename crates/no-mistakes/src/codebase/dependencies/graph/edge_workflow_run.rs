struct WorkflowRunResolver<'a> {
    root: PathBuf,
    universe: &'a HashSet<PathBuf>,
    cargo_bins: &'a CargoBinIndex,
    package_scripts: HashMap<PathBuf, Option<HashMap<String, String>>>,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
}

impl<'a> WorkflowRunResolver<'a> {
    fn new(
        root: &'a Path,
        universe: &'a HashSet<PathBuf>,
        cargo_bins: &'a CargoBinIndex,
        sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    ) -> Self {
        Self {
            root: crate::codebase::ts_resolver::normalize_path(root),
            universe,
            cargo_bins,
            package_scripts: HashMap::new(),
            sources,
        }
    }

    fn resolve(&mut self, run: &str, working_directory: &Path) -> Vec<PathBuf> {
        let mut targets = HashSet::new();
        let mut script_stack = HashSet::new();
        self.resolve_commands(run, working_directory, &mut script_stack, &mut targets);
        let mut targets: Vec<PathBuf> = targets.into_iter().collect();
        targets.sort();
        targets
    }

    fn resolve_commands(
        &mut self,
        source: &str,
        working_directory: &Path,
        script_stack: &mut HashSet<(PathBuf, String)>,
        targets: &mut HashSet<PathBuf>,
    ) {
        for mut words in static_command_segments(source) {
            while words
                .first()
                .is_some_and(|word| is_environment_assignment(word))
            {
                words.remove(0);
            }
            if words.is_empty() {
                continue;
            }
            self.resolve_cargo_targets(&words, targets);
            if let Some(script) = package_script_command(&words) {
                self.resolve_package_script(script, working_directory, script_stack, targets);
                continue;
            }
            if let Some(script) = interpreter_script(&words) {
                self.insert_local_path(script, working_directory, targets);
                continue;
            }
            let command = &words[0];
            if command.contains('/') {
                self.insert_local_path(command, working_directory, targets);
            }
        }
    }

    fn resolve_cargo_targets(&self, words: &[String], targets: &mut HashSet<PathBuf>) {
        let Some(command) = words.first() else {
            return;
        };
        let is_cargo = command == "cargo";
        let is_built_binary = command.starts_with("./target/") || command.starts_with("target/");
        if !is_cargo && !is_built_binary {
            return;
        }
        let source = words.join(" ");
        let cargo_targets = crate::codebase::ci_workflows::extract_cargo_targets(&source);
        for target in &cargo_targets {
            if let Some(path) = self.cargo_bins.get_cargo_target(target) {
                targets.insert(path.clone());
            }
        }
        for binary in crate::codebase::ci_workflows::extract_binary_names(&source) {
            if cargo_targets.iter().any(|target| target.binary == binary) {
                continue;
            }
            if let Some(path) = self.cargo_bins.by_name.get(&binary) {
                targets.insert(path.clone());
            }
        }
    }

    fn resolve_package_script(
        &mut self,
        script: &str,
        working_directory: &Path,
        script_stack: &mut HashSet<(PathBuf, String)>,
        targets: &mut HashSet<PathBuf>,
    ) {
        if !is_static_path_token(script) {
            return;
        }
        let Some(manifest) = self.nearest_package_json(working_directory) else {
            return;
        };
        targets.insert(manifest.clone());
        let key = (manifest.clone(), script.to_string());
        if !script_stack.insert(key.clone()) {
            return;
        }
        let command = self
            .scripts_for(&manifest)
            .and_then(|scripts| scripts.get(script).cloned());
        if let (Some(command), Some(directory)) = (command, manifest.parent()) {
            self.resolve_commands(&command, directory, script_stack, targets);
        }
        script_stack.remove(&key);
    }

    fn nearest_package_json(&self, working_directory: &Path) -> Option<PathBuf> {
        let mut directory = crate::codebase::ts_resolver::normalize_path(working_directory);
        loop {
            let manifest =
                crate::codebase::ts_resolver::normalize_path(&directory.join("package.json"));
            if self.universe.contains(&manifest) {
                return Some(manifest);
            }
            if directory == self.root || !directory.pop() || !directory.starts_with(&self.root) {
                return None;
            }
        }
    }

    fn scripts_for(&mut self, manifest: &Path) -> Option<&HashMap<String, String>> {
        if !self.package_scripts.contains_key(manifest) {
            let scripts = crate::codebase::ts_source::SourceStore::parse_json_optional(
                self.sources,
                manifest,
            )
            .and_then(|value| value.get("scripts")?.as_object().cloned())
            .map(|scripts| {
                scripts
                    .into_iter()
                    .filter_map(|(name, value)| Some((name, value.as_str()?.to_string())))
                    .collect()
            });
            self.package_scripts.insert(manifest.to_path_buf(), scripts);
        }
        self.package_scripts.get(manifest).and_then(Option::as_ref)
    }

    fn insert_local_path(
        &self,
        raw: &str,
        working_directory: &Path,
        targets: &mut HashSet<PathBuf>,
    ) {
        if !is_static_path_token(raw) {
            return;
        }
        let raw = Path::new(raw);
        let path = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            working_directory.join(raw)
        };
        let path = crate::codebase::ts_resolver::normalize_path(&path);
        if path.starts_with(&self.root) && self.universe.contains(&path) {
            targets.insert(path);
        }
    }
}

include!("edge_workflow_run_cwd.rs");
