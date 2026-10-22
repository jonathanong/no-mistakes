fn load_includes(
    root: &Path,
    root_map: &serde_yaml::Mapping,
    read: &mut impl FnMut(&Path) -> Option<std::sync::Arc<str>>,
    docs: &mut Vec<Doc>,
    phantoms: &mut Vec<Phantom>,
    load_errors: &mut Vec<IncludeLoadError>,
) {
    let Some(include) = root_map.get("include") else {
        return;
    };
    let Some(mapping) = include.as_mapping() else {
        load_errors.push(IncludeLoadError {
            message: "pnpm include must be a mapping of local YAML files".to_string(),
            settings: vec!["*".to_string()],
        });
        // The grant list is unreadable, so every setting stays reserved.
        phantoms.push(Phantom {
            label: "include".to_string(),
            kind: DocKind::Star,
            listed: Vec::new(),
        });
        return;
    };
    for (key, grant) in mapping {
        let Some(source) = key.as_str() else {
            continue;
        };
        if !(source.starts_with("./") || source.starts_with("../")) {
            continue;
        }
        let Some((kind, listed)) = parse_grant(grant) else {
            load_errors.push(IncludeLoadError {
                message: format!("included source \"{source}\" has an invalid grant"),
                settings: vec!["*".to_string()],
            });
            phantoms.push(Phantom {
                label: source.to_string(),
                kind: DocKind::Star,
                listed: Vec::new(),
            });
            continue;
        };
        let settings = grant_settings(kind, &listed);
        if !local_yaml_source(source) {
            reject_include(
                source,
                kind,
                listed,
                settings,
                "is not a local YAML file",
                phantoms,
                load_errors,
            );
            continue;
        }
        let path = normalize_path(&root.join(source));
        if !path.starts_with(root) {
            reject_include(
                source,
                kind,
                listed,
                settings,
                "escapes the workspace root",
                phantoms,
                load_errors,
            );
            continue;
        }
        let Some(text) = read(&path) else {
            reject_include(
                source,
                kind,
                listed,
                settings,
                "could not be read",
                phantoms,
                load_errors,
            );
            continue;
        };
        let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(&text) else {
            reject_include(
                source,
                kind,
                listed,
                settings,
                "is not valid YAML",
                phantoms,
                load_errors,
            );
            continue;
        };
        let Some(body) = parsed.as_mapping().cloned() else {
            reject_include(
                source,
                kind,
                listed,
                settings,
                "must be a mapping",
                phantoms,
                load_errors,
            );
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let dir_rel = rel
            .rsplit_once('/')
            .map(|(dir, _)| dir.to_string())
            .unwrap_or_default();
        docs.push(Doc {
            label: source.to_string(),
            path: path.clone(),
            base_dir: path.parent().unwrap_or(root).to_path_buf(),
            dir_rel,
            body,
            kind,
            listed,
        });
    }
}

fn reject_include(
    source: &str,
    kind: DocKind,
    listed: Vec<String>,
    settings: Vec<String>,
    reason: &str,
    phantoms: &mut Vec<Phantom>,
    load_errors: &mut Vec<IncludeLoadError>,
) {
    load_errors.push(IncludeLoadError {
        message: format!("included file \"{source}\" {reason}"),
        settings,
    });
    phantoms.push(Phantom {
        label: source.to_string(),
        kind,
        listed,
    });
}

fn local_yaml_source(source: &str) -> bool {
    (source.ends_with(".yaml") || source.ends_with(".yml"))
        && !source.contains(['*', '?', '[', ']'])
}

fn parse_grant(value: &serde_yaml::Value) -> Option<(DocKind, Vec<String>)> {
    if value.as_str() == Some("*") {
        return Some((DocKind::Star, Vec::new()));
    }
    let seq = value.as_sequence()?;
    let mut listed = Vec::new();
    for item in seq {
        listed.push(item.as_str()?.to_string());
    }
    Some((DocKind::Listed, listed))
}

fn grant_settings(kind: DocKind, listed: &[String]) -> Vec<String> {
    if kind == DocKind::Star {
        vec!["*".to_string()]
    } else {
        listed.to_vec()
    }
}
