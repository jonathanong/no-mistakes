impl CatalogBuilder<'_> {
    fn load_effective(&mut self, path: &Path) -> Result<EffectiveConfig, String> {
        let path = normalize_path(path);
        let identity = match real_path(&path) {
            Some(path) => path,
            None => return Err(format!("cannot resolve {}", path.display())),
        };
        if let Some(state) = self.states.get(&path) {
            return state.clone();
        }
        if !self.loading.insert(identity.clone()) {
            return Err(format!("tsconfig.extends cycle detected at {}", path.display()));
        }
        let result = self.parse_effective(&path);
        self.loading.remove(&identity);
        self.states.insert(path, result.clone());
        result
    }

    fn parse_effective(&mut self, path: &Path) -> Result<EffectiveConfig, String> {
        let dir = match path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return Err(format!("resolving parent directory for {}", path.display())),
        };
        let content = match self.sources {
            Some(sources) => match sources.read_path(path) {
                Ok(content) => content,
                Err(error) => return Err(format!("reading {}: {error}", path.display())),
            },
            None => match std::fs::read_to_string(path) {
                Ok(content) => std::sync::Arc::<str>::from(content),
                Err(error) => return Err(format!("reading {}: {error}", path.display())),
            },
        };
        let value: Option<serde_json::Value> = match jsonc_parser::parse_to_serde_value(
            &content,
            &jsonc_parser::ParseOptions::default(),
        ) {
            Ok(value) => value,
            Err(error) => return Err(format!("parsing {}: {error}", path.display())),
        };
        let value = value.unwrap_or(serde_json::Value::Null);
        let mut effective = EffectiveConfig::new(path.to_path_buf(), dir.clone());
        for extends in extends_values(&value, path)? {
            let package_extends = is_package_extends(&dir, &extends);
            let base = match self.resolve_extends(&dir, &extends) {
                Ok(base) => base,
                Err(error) if package_extends => {
                    self.invalid_config(path, error);
                    continue;
                }
                Err(error) => return Err(error),
            };
            let base = match self.load_effective(&base) {
                Ok(base) => base,
                Err(error) if package_extends => {
                    self.invalid_config(
                        path,
                        format!("loading extended tsconfig {}: {error}", base.display()),
                    );
                    continue;
                }
                Err(error) => {
                    return Err(format!("loading extended tsconfig {}: {error}", base.display()));
                }
            };
            effective.inherit(base);
        }
        effective.apply_own(&value, path, &dir, |reference| self.resolve_config_value(&dir, reference))?;
        Ok(effective)
    }

    fn queue_references(
        &mut self,
        path: &Path,
        references: &[PathBuf],
        referenced_configs: &mut BTreeSet<PathBuf>,
        pending: &mut Vec<PathBuf>,
    ) {
        for reference in references {
            let lexical = normalize_path(reference);
            match real_path(&lexical) {
                Some(reference) if !reference.starts_with(&self.root_real) => {
                    self.diagnostics.insert(TsConfigDiagnostic::config(
                        TsConfigDiagnosticKind::InvalidReference,
                        path,
                        format!("referenced config {} is outside configured analysis roots", lexical.display()),
                    ));
                }
                Some(_) if !self.is_visible(&lexical) => {
                    self.diagnostics.insert(TsConfigDiagnostic::config(
                        TsConfigDiagnosticKind::InvalidReference,
                        path,
                        format!("referenced config {} is not in the visible analysis paths", lexical.display()),
                    ));
                }
                Some(_) => {
                    referenced_configs.insert(lexical.clone());
                    pending.push(lexical);
                }
                None => {
                    self.diagnostics.insert(TsConfigDiagnostic::config(
                        TsConfigDiagnosticKind::InvalidReference,
                        path,
                        format!("referenced config {} does not exist", lexical.display()),
                    ));
                }
            };
        }
    }
}

fn is_package_extends(dir: &Path, value: &str) -> bool {
    let value = expand_config_dir(value, dir);
    !value.starts_with('.') && !Path::new(&value).is_absolute()
}
