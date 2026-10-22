fn root_package_list(map: &serde_yaml::Mapping) -> Vec<String> {
    map.get("packages")
        .and_then(yaml_string_list)
        .unwrap_or_default()
}

fn packages_field_invalid(map: &serde_yaml::Mapping) -> bool {
    match map.get("packages") {
        None => false,
        Some(value) if value.is_null() => false,
        Some(value) => yaml_string_list(value).is_none(),
    }
}

fn yaml_string_list(value: &serde_yaml::Value) -> Option<Vec<String>> {
    let seq = value.as_sequence()?;
    let mut out = Vec::with_capacity(seq.len());
    for item in seq {
        out.push(item.as_str()?.to_string());
    }
    Some(out)
}

fn setting_defined(value: Option<&serde_yaml::Value>) -> bool {
    value.is_some_and(|value| !value.is_null())
}

enum Winner<'a> {
    Absent,
    Missing,
    Conflict,
    Doc(&'a Doc),
}

fn winner<'a>(
    docs: &'a [Doc],
    phantoms: &[Phantom],
    mut claims: impl FnMut(&Doc) -> bool,
    mut phantom_claims: impl FnMut(&Phantom) -> bool,
    duplicates: &mut Vec<String>,
    duplicate_label: &str,
) -> Winner<'a> {
    let mut labels = Vec::new();
    for doc in docs {
        if claims(doc) {
            labels.push(doc.label.as_str());
        }
    }
    for phantom in phantoms {
        if phantom_claims(phantom) {
            labels.push(phantom.label.as_str());
        }
    }
    match labels.len() {
        0 => Winner::Absent,
        1 => docs
            .iter()
            .find(|doc| doc.label == labels[0])
            .map(Winner::Doc)
            .unwrap_or(Winner::Missing),
        _ => {
            duplicates.push(duplicate_label.to_string());
            Winner::Conflict
        }
    }
}

fn claims_simple(doc: &Doc, key: &str) -> bool {
    match doc.kind {
        DocKind::Listed => doc.listed.iter().any(|item| item == key),
        DocKind::Root | DocKind::Star => setting_defined(doc.body.get(key)),
    }
}

fn phantom_claims_simple(phantom: &Phantom, key: &str) -> bool {
    match phantom.kind {
        DocKind::Listed => phantom.listed.iter().any(|item| item == key),
        DocKind::Star => true,
        DocKind::Root => false,
    }
}

fn package_globs(docs: &[Doc], phantoms: &[Phantom], duplicates: &mut Vec<String>) -> PackageGlobs {
    match winner(
        docs,
        phantoms,
        |doc| claims_simple(doc, "packages"),
        |phantom| phantom_claims_simple(phantom, "packages"),
        duplicates,
        "packages",
    ) {
        Winner::Absent => PackageGlobs::DefaultStar,
        Winner::Missing => PackageGlobs::List(Vec::new()),
        Winner::Conflict => PackageGlobs::Conflict,
        Winner::Doc(doc) => match doc.body.get("packages") {
            None => PackageGlobs::List(Vec::new()),
            Some(value) if value.is_null() => PackageGlobs::List(Vec::new()),
            Some(value) => match yaml_string_list(value) {
                Some(list) if doc.kind == DocKind::Root => PackageGlobs::List(list),
                Some(list) => PackageGlobs::List(
                    list.into_iter()
                        .filter_map(|pattern| rebase_glob(&pattern, &doc.dir_rel))
                        .collect(),
                ),
                None => PackageGlobs::List(Vec::new()),
            },
        },
    }
}

fn override_paths(
    docs: &[Doc],
    phantoms: &[Phantom],
    duplicates: &mut Vec<String>,
) -> Vec<PathBuf> {
    let won = winner(
        docs,
        phantoms,
        |doc| claims_simple(doc, "overrides"),
        |phantom| phantom_claims_simple(phantom, "overrides"),
        duplicates,
        "overrides",
    );
    let conflict = matches!(won, Winner::Conflict);
    let winner_label = match won {
        Winner::Doc(doc) => Some(doc.label.clone()),
        _ => None,
    };
    docs.iter()
        .filter(|doc| doc.kind != DocKind::Root && setting_defined(doc.body.get("overrides")))
        .filter(|doc| conflict || winner_label.as_deref() == Some(doc.label.as_str()))
        .map(|doc| doc.path.clone())
        .collect()
}

fn release_age(
    docs: &[Doc],
    phantoms: &[Phantom],
    duplicates: &mut Vec<String>,
) -> ReleaseAgeExclude {
    match winner(
        docs,
        phantoms,
        |doc| claims_simple(doc, "minimumReleaseAgeExclude"),
        |phantom| phantom_claims_simple(phantom, "minimumReleaseAgeExclude"),
        duplicates,
        "minimumReleaseAgeExclude",
    ) {
        Winner::Doc(doc) if doc.kind != DocKind::Root => doc
            .body
            .get("minimumReleaseAgeExclude")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| ReleaseAgeExclude::Included(seq.clone()))
            .unwrap_or(ReleaseAgeExclude::Included(Vec::new())),
        Winner::Conflict => ReleaseAgeExclude::Conflict,
        Winner::Missing => ReleaseAgeExclude::Included(Vec::new()),
        Winner::Absent | Winner::Doc(_) => ReleaseAgeExclude::FromRoot,
    }
}

fn rebase_glob(pattern: &str, dir_rel: &str) -> Option<String> {
    let negated = pattern.starts_with('!');
    let body = pattern.strip_prefix('!').unwrap_or(pattern);
    if body.is_empty() || body.starts_with('/') || body.starts_with('~') || body.starts_with('$') {
        return None;
    }
    let combined = if dir_rel.is_empty() {
        body.to_string()
    } else {
        format!("{dir_rel}/{body}")
    };
    let normalized = glob_normalize::normalize(&combined);
    if normalized.is_empty() || normalized.split('/').any(|part| part == "..") {
        return None;
    }
    Some(if negated {
        format!("!{normalized}")
    } else {
        normalized
    })
}
