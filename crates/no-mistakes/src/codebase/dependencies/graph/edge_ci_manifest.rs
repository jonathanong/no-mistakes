fn cargo_member_globset(members: &[String]) -> Option<globset::GlobSet> {
    if members.is_empty() {
        return None;
    }
    let mut builder = globset::GlobSetBuilder::new();
    for member in members {
        let glob = globset::GlobBuilder::new(member)
            .literal_separator(true)
            .build()
            .ok()?;
        builder.add(glob);
    }
    builder.build().ok()
}

fn add_manifest_bins(
    manifest: &Path,
    cargo_toml: &str,
    visible_files: &crate::fx::PathSet,
    bins: &mut CargoBinIndex,
) {
    let Ok(parsed_bins) = crate::codebase::ci_workflows::parse_cargo_bins(cargo_toml) else {
        return;
    };
    let package = crate::codebase::ci_workflows::parse_cargo_package_name(cargo_toml)
        .ok()
        .flatten();
    let Some(manifest_dir) = manifest.parent() else {
        return;
    };
    for (name, rel_path) in parsed_bins {
        if let Some(source_file) =
            resolve_cargo_bin_source(manifest_dir, &name, &rel_path, visible_files)
        {
            bins.insert(package.as_deref(), name, source_file);
        }
    }
}

fn resolve_cargo_bin_source(
    manifest_dir: &Path,
    name: &str,
    rel_path: &str,
    visible_files: &crate::fx::PathSet,
) -> Option<PathBuf> {
    let declared = crate::codebase::ts_resolver::normalize_path(&manifest_dir.join(rel_path));
    if visible_files.contains(&declared) {
        return Some(declared);
    }

    let nested = manifest_dir
        .join("src")
        .join("bin")
        .join(name)
        .join("main.rs");
    if visible_files.contains(&nested) {
        return Some(nested);
    }

    None
}
