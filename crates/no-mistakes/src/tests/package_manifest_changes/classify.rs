use std::collections::BTreeSet;

const DEPENDENCY_FIELDS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PackageManifestChange {
    DependencyOnly(BTreeSet<String>),
    FormattingOnly,
    Broad,
}

pub(super) fn classify_change(before: &str, after: &str) -> Result<PackageManifestChange, ()> {
    let mut before = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(before)
        .map_err(|_| ())?;
    let mut after = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(after)
        .map_err(|_| ())?;
    for manifest in [&before, &after] {
        if DEPENDENCY_FIELDS.iter().any(|field| {
            manifest
                .get(*field)
                .is_some_and(|dependencies| !dependencies.is_object())
        }) {
            return Err(());
        }
    }
    if before == after {
        return Ok(PackageManifestChange::FormattingOnly);
    }
    let mut changed_packages = BTreeSet::new();
    for field in DEPENDENCY_FIELDS {
        let before_dependencies = before.get(field).and_then(serde_json::Value::as_object);
        let after_dependencies = after.get(field).and_then(serde_json::Value::as_object);
        let names = before_dependencies
            .into_iter()
            .flat_map(|dependencies| dependencies.keys())
            .chain(
                after_dependencies
                    .into_iter()
                    .flat_map(|dependencies| dependencies.keys()),
            );
        for name in names {
            if before_dependencies.and_then(|dependencies| dependencies.get(name))
                != after_dependencies.and_then(|dependencies| dependencies.get(name))
            {
                changed_packages.insert(name.clone());
            }
        }
    }
    for field in DEPENDENCY_FIELDS {
        before.remove(field);
        after.remove(field);
    }
    Ok(if !changed_packages.is_empty() && before == after {
        PackageManifestChange::DependencyOnly(changed_packages)
    } else {
        PackageManifestChange::Broad
    })
}
