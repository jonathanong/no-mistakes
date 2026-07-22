fn extends_values(value: &serde_json::Value, path: &Path) -> Result<Vec<String>, String> {
    match value.get("extends") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::String(value)) => Ok(vec![value.clone()]),
        Some(serde_json::Value::Array(values)) => values.iter().map(|value| value.as_str()
            .map(ToString::to_string)
            .ok_or_else(|| format!("{} extends array contains a non-string entry; TypeScript rejects this configuration", path.display()))).collect(),
        Some(value) => Err(format!("{} extends must be a string or array of strings, got {value}", path.display())),
    }
}

fn parse_paths(value: &serde_json::Value, dir: &Path) -> Result<Vec<(String, Vec<String>)>, String> {
    let paths = value.as_object().ok_or_else(|| "compilerOptions.paths must be an object".to_string())?;
    paths.iter().map(|(pattern, replacements)| {
        let replacements = replacements.as_array().ok_or_else(|| {
            format!("compilerOptions.paths.{pattern} must be an array of strings")
        })?;
        let parsed = replacements.iter().map(|replacement| replacement.as_str()
            .map(|replacement| expand_config_dir(replacement, dir))
            .ok_or_else(|| format!("compilerOptions.paths.{pattern} must be an array of strings")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((pattern.clone(), parsed))
    }).collect()
}

fn config_relative_path(value: &serde_json::Value, dir: &Path, label: &str) -> Result<PathBuf, String> {
    let value = value.as_str().ok_or_else(|| format!("{label} must be a string"))?;
    let value = PathBuf::from(expand_config_dir(value, dir));
    Ok(if value.is_absolute() { normalize_path(&value) } else { normalize_path(&dir.join(value)) })
}

fn patterns(value: &serde_json::Value, path: &Path, label: &str, dir: &Path) -> Result<Vec<PatternInput>, String> {
    Ok(string_list(value, path, label)?.into_iter().map(|value| PatternInput {
        base: dir.to_path_buf(), value: expand_config_dir(&value, dir),
    }).collect())
}

fn string_list(value: &serde_json::Value, path: &Path, label: &str) -> Result<Vec<String>, String> {
    let values = value.as_array().ok_or_else(|| format!("{} {label} must be an array of strings", path.display()))?;
    values.iter().map(|value| value.as_str().map(ToString::to_string)
        .ok_or_else(|| format!("{} {label} must be an array of strings", path.display()))).collect()
}

fn reference_values(value: &serde_json::Value, path: &Path) -> Result<Vec<String>, String> {
    let values = value.as_array().ok_or_else(|| format!("{} references must be an array", path.display()))?;
    values.iter().map(|value| match value {
        serde_json::Value::String(value) => Ok(value.clone()),
        serde_json::Value::Object(value) => value.get("path").and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| format!("{} references entries require a string path", path.display())),
        _ => Err(format!("{} references entries require a path", path.display())),
    }).collect()
}

fn default_excludes(dir: &Path) -> Vec<PatternInput> {
    ["node_modules", "bower_components", "jspm_packages"].into_iter().map(|value| PatternInput {
        base: dir.to_path_buf(), value: value.to_string(),
    }).collect()
}

fn expand_config_dir(value: &str, dir: &Path) -> String {
    value.replace("${configDir}", &dir.to_string_lossy())
}
