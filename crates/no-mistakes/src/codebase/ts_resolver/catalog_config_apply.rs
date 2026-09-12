impl EffectiveConfig {
    // llvm-cov counts `?` Ok/Err as extra regions even when both paths are
    // tested; keep explicit matches so the catalog apply helper can hit 99%.
    #[inline(never)]
    #[allow(clippy::question_mark)]
    fn apply_own(
        &mut self,
        value: &serde_json::Value,
        path: &Path,
        dir: &Path,
        resolve_reference: impl Fn(&str) -> Result<PathBuf, String>,
    ) -> Result<(), String> {
        let compiler = value.get("compilerOptions").and_then(serde_json::Value::as_object);
        if let Some(compiler) = compiler {
            if let Some(paths) = compiler.get("paths") {
                match parse_paths(paths, dir) {
                    Ok(parsed) => self.paths = Some((parsed, dir.to_path_buf())),
                    Err(error) => return Err(error),
                }
            }
            if let Some(base_url) = compiler.get("baseUrl") {
                match config_relative_path(base_url, dir, "compilerOptions.baseUrl") {
                    Ok(parsed) => self.base_url = Some(parsed),
                    Err(error) => return Err(error),
                }
            }
            if let Some(allow_js) = compiler.get("allowJs") {
                match allow_js.as_bool() {
                    Some(parsed) => self.allow_js = Some(parsed),
                    None => {
                        return Err(format!(
                            "{} compilerOptions.allowJs must be a boolean",
                            path.display()
                        ));
                    }
                }
            }
            if let Some(out_dir) = compiler.get("outDir") {
                match config_relative_path(out_dir, dir, "compilerOptions.outDir") {
                    Ok(parsed) => self.out_dir = Some(parsed),
                    Err(error) => return Err(error),
                }
            }
            if let Some(module_resolution) = compiler.get("moduleResolution") {
                match module_resolution.as_str() {
                    Some(parsed) => {
                        self.module_resolution = Some(parsed.to_ascii_lowercase());
                    }
                    None => {
                        return Err(format!(
                            "{} compilerOptions.moduleResolution must be a string",
                            path.display()
                        ));
                    }
                }
            }
        } else if value.get("compilerOptions").is_some() {
            return Err(format!("{} compilerOptions must be an object", path.display()));
        }
        if let Some(files) = value.get("files") {
            match string_list(files, path, "files") {
                Ok(parsed) => {
                    self.files = Some(
                        parsed
                            .into_iter()
                            .map(|file| normalize_path(&dir.join(expand_config_dir(&file, dir))))
                            .collect(),
                    );
                }
                Err(error) => return Err(error),
            }
        }
        if let Some(includes) = value.get("include") {
            match patterns(includes, path, "include", dir) {
                Ok(parsed) => self.includes = Some(parsed),
                Err(error) => return Err(error),
            }
        }
        if let Some(excludes) = value.get("exclude") {
            match patterns(excludes, path, "exclude", dir) {
                Ok(parsed) => self.excludes = Some(parsed),
                Err(error) => return Err(error),
            }
        }
        if let Some(references) = value.get("references") {
            let parsed = match reference_values(references, path) {
                Ok(parsed) => parsed,
                Err(error) => return Err(error),
            };
            let mut resolved = Vec::new();
            for reference in parsed {
                match resolve_reference(&reference) {
                    Ok(path) => resolved.push(path),
                    Err(error) => return Err(error),
                }
            }
            self.references = resolved;
        }
        Ok(())
    }
}
