impl EffectiveConfig {
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
                self.paths = Some((parse_paths(paths, dir)?, dir.to_path_buf()));
            }
            if let Some(base_url) = compiler.get("baseUrl") {
                self.base_url = Some(config_relative_path(base_url, dir, "compilerOptions.baseUrl")?);
            }
            if let Some(allow_js) = compiler.get("allowJs") {
                self.allow_js = Some(allow_js.as_bool().ok_or_else(|| {
                    format!("{} compilerOptions.allowJs must be a boolean", path.display())
                })?);
            }
            if let Some(out_dir) = compiler.get("outDir") {
                self.out_dir = Some(config_relative_path(out_dir, dir, "compilerOptions.outDir")?);
            }
            if let Some(module_resolution) = compiler.get("moduleResolution") {
                self.module_resolution = Some(module_resolution.as_str().ok_or_else(|| {
                    format!("{} compilerOptions.moduleResolution must be a string", path.display())
                })?.to_ascii_lowercase());
            }
        } else if value.get("compilerOptions").is_some() {
            return Err(format!("{} compilerOptions must be an object", path.display()));
        }
        if let Some(files) = value.get("files") {
            self.files = Some(string_list(files, path, "files")?.into_iter().map(|file| {
                normalize_path(&dir.join(expand_config_dir(&file, dir)))
            }).collect());
        }
        if let Some(includes) = value.get("include") {
            self.includes = Some(patterns(includes, path, "include", dir)?);
        }
        if let Some(excludes) = value.get("exclude") {
            self.excludes = Some(patterns(excludes, path, "exclude", dir)?);
        }
        if let Some(references) = value.get("references") {
            self.references = reference_values(references, path)?.into_iter()
                .map(|reference| resolve_reference(&reference)).collect::<Result<Vec<_>, _>>()?;
        }
        Ok(())
    }
}
