impl CatalogBuilder<'_> {
    fn resolve_extends(&self, dir: &Path, raw: &str) -> Result<PathBuf, String> {
        let raw = expand_config_dir(raw, dir);
        if raw.starts_with('.') || Path::new(&raw).is_absolute() {
            self.resolve_config_value(dir, &raw)
        } else {
            self.resolve_package_extends(dir, &raw)
        }
    }

    fn resolve_config_value(&self, dir: &Path, raw: &str) -> Result<PathBuf, String> {
        let raw = expand_config_dir(raw, dir);
        let candidate = PathBuf::from(&raw);
        let candidate = if candidate.is_absolute() { candidate } else { dir.join(candidate) };
        let candidate = normalize_path(&candidate);
        Ok(if candidate.is_dir() {
            candidate.join("tsconfig.json")
        } else if candidate.is_file()
            || candidate.extension() == Some(std::ffi::OsStr::new("json"))
        {
            candidate
        } else {
            let mut json = candidate.into_os_string();
            json.push(".json");
            PathBuf::from(json)
        })
    }

    fn resolve_package_extends(&self, dir: &Path, raw: &str) -> Result<PathBuf, String> {
        let mut current = Some(dir);
        while let Some(base) = current {
            let candidate = base.join("node_modules").join(raw);
            if candidate.exists() {
                if candidate.is_dir() {
                    let package = candidate.join("package.json");
                    if let Ok(content) = std::fs::read_to_string(&package) {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(tsconfig) = value.get("tsconfig") {
                                if let Some(tsconfig) = tsconfig.as_str() {
                                    return self.resolve_config_value(&candidate, tsconfig);
                                }
                            }
                        }
                    }
                    return Ok(candidate.join("tsconfig.json"));
                }
                return Ok(candidate);
            }
            let json = candidate.with_extension("json");
            if json.exists() {
                return Ok(json);
            }
            current = base.parent();
        }
        Err(format!("cannot resolve npm tsconfig package '{raw}' from {}", dir.display()))
    }

    fn invalid_config(&mut self, path: &Path, detail: String) {
        let kind = if detail.contains("extend") || detail.contains("npm tsconfig") {
            TsConfigDiagnosticKind::InvalidExtends
        } else {
            TsConfigDiagnosticKind::InvalidConfig
        };
        self.diagnostics.insert(TsConfigDiagnostic::config(kind, path, detail));
    }
}
