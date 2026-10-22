fn fill_catalogs(docs: &[Doc], phantoms: &[Phantom], duplicates: &mut Vec<String>) -> CatalogIndex {
    let mut slots = std::collections::BTreeSet::new();
    slots.insert(String::new());
    for doc in docs {
        push_catalog_slots(&mut slots, doc.body.get("catalogs"));
        if doc.kind == DocKind::Listed {
            push_listed_slots(&mut slots, &doc.listed);
        }
    }
    for phantom in phantoms {
        if phantom.kind == DocKind::Listed {
            push_listed_slots(&mut slots, &phantom.listed);
        }
    }
    if docs
        .iter()
        .any(|doc| doc.kind == DocKind::Root && doc.body.contains_key("catalogs"))
        && grants_whole_catalogs(docs, phantoms)
    {
        duplicates.push("catalogs".to_string());
    }
    let mut index = CatalogIndex::default();
    for slot in slots {
        let label = if slot.is_empty() {
            "catalog".to_string()
        } else {
            format!("catalogs.{slot}")
        };
        match winner(
            docs,
            phantoms,
            |doc| claims_slot(doc, &slot),
            |phantom| phantom_claims_slot(phantom, &slot),
            duplicates,
            &label,
        ) {
            Winner::Doc(doc) => {
                let hits = catalog_hits(doc, &slot);
                if slot.is_empty() {
                    index.default_entries = hits;
                } else {
                    index.named.insert(slot, hits);
                }
            }
            Winner::Absent | Winner::Missing | Winner::Conflict => {}
        }
    }
    index
}

fn push_catalog_slots(
    slots: &mut std::collections::BTreeSet<String>,
    value: Option<&serde_yaml::Value>,
) {
    let Some(mapping) = value.and_then(serde_yaml::Value::as_mapping) else {
        return;
    };
    for (key, _) in mapping {
        if let Some(name) = key.as_str() {
            if name != "default" {
                slots.insert(name.to_string());
            }
        }
    }
}

fn push_listed_slots(slots: &mut std::collections::BTreeSet<String>, listed: &[String]) {
    for key in listed {
        if let Some(name) = key.strip_prefix("catalogs.") {
            if !name.is_empty() && name != "default" && !name.contains('.') {
                slots.insert(name.to_string());
            }
        }
    }
}

fn grants_whole_catalogs(docs: &[Doc], phantoms: &[Phantom]) -> bool {
    docs.iter()
        .any(|doc| doc.listed.iter().any(|key| key == "catalogs"))
        || phantoms
            .iter()
            .any(|phantom| phantom.listed.iter().any(|key| key == "catalogs"))
}

fn claims_slot(doc: &Doc, slot: &str) -> bool {
    match doc.kind {
        DocKind::Listed => doc.listed.iter().any(|key| key_claims_slot(key, slot)),
        DocKind::Root | DocKind::Star => defines_slot(doc, slot),
    }
}

fn phantom_claims_slot(phantom: &Phantom, slot: &str) -> bool {
    match phantom.kind {
        DocKind::Listed => phantom.listed.iter().any(|key| key_claims_slot(key, slot)),
        DocKind::Star => true,
        DocKind::Root => false,
    }
}

fn key_claims_slot(key: &str, slot: &str) -> bool {
    if key == "catalogs" {
        return true;
    }
    if slot.is_empty() {
        key == "catalog" || key == "catalogs.default"
    } else {
        key.strip_prefix("catalogs.")
            .is_some_and(|name| name == slot)
    }
}

fn defines_slot(doc: &Doc, slot: &str) -> bool {
    if slot.is_empty() {
        return doc.body.contains_key("catalog")
            || doc
                .body
                .get("catalogs")
                .and_then(serde_yaml::Value::as_mapping)
                .is_some_and(|mapping| mapping.contains_key("default"));
    }
    doc.body
        .get("catalogs")
        .and_then(serde_yaml::Value::as_mapping)
        .is_some_and(|mapping| mapping.contains_key(slot))
}

fn catalog_hits(doc: &Doc, slot: &str) -> std::collections::BTreeMap<String, CatalogHit> {
    let mut entries = Vec::new();
    if slot.is_empty() {
        if let Some(value) = doc.body.get("catalog") {
            push_string_map(&mut entries, value);
        }
        if let Some(value) = doc
            .body
            .get("catalogs")
            .and_then(|value| value.get("default"))
        {
            push_string_map(&mut entries, value);
        }
    } else if let Some(value) = doc.body.get("catalogs").and_then(|value| value.get(slot)) {
        push_string_map(&mut entries, value);
    }
    entries
        .into_iter()
        .map(|(name, specifier)| {
            (
                name,
                CatalogHit {
                    specifier,
                    base_dir: doc.base_dir.clone(),
                },
            )
        })
        .collect()
}

fn push_string_map(entries: &mut Vec<(String, String)>, value: &serde_yaml::Value) {
    let Some(mapping) = value.as_mapping() else {
        return;
    };
    for (key, spec) in mapping {
        let (Some(name), Some(spec)) = (key.as_str(), spec.as_str()) else {
            continue;
        };
        if entries.iter().any(|(existing, _)| existing == name) {
            continue;
        }
        entries.push((name.to_string(), spec.to_string()));
    }
}
