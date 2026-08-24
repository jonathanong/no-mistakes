fn collect_swift_package_edges(
    facts: &crate::codebase::swift::SwiftFactMap,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for package in &facts.packages {
        for target in package.targets.values() {
            let source_files: Vec<_> = facts
                .files
                .values()
                .filter(|file| {
                    swift_owning_package(facts, &file.path)
                        .is_some_and(|owner| owner.package_root == package.package_root)
                        && file.target.as_deref() == Some(&target.name)
                })
                .collect();
            if source_files.is_empty() {
                continue;
            }
            for dependency in &target.dependencies {
                let dependency_targets = swift_dependency_targets(facts, package, target, dependency);
                let dep_files: Vec<_> = facts
                    .files
                    .values()
                    .filter(|file| {
                        dependency_targets.iter().any(|(root, names)| {
                            swift_owning_package(facts, &file.path)
                                .is_some_and(|owner| owner.package_root == *root)
                                && file.target.as_ref().is_some_and(|name| names.contains(name))
                        })
                    })
                    .collect();
                for source in &source_files {
                    for dependency in &dep_files {
                        if source.path != dependency.path {
                            edges.push((
                                NodeId::file_in(interner, &source.path),
                                NodeId::file_in(interner, &dependency.path),
                                EdgeKind::SwiftPackageDependency,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn swift_dependency_roots(
    facts: &crate::codebase::swift::SwiftFactMap,
    source_package: &crate::codebase::swift::SwiftPackageFacts,
    target: &crate::codebase::swift::SwiftTargetFacts,
) -> Vec<PathBuf> {
    let mut roots = vec![source_package.package_root.clone()];
    for dependency in &target.dependencies {
        roots.extend(
            swift_dependency_packages(facts, source_package, target, dependency)
                .into_iter()
                .map(|package| package.package_root.clone()),
        );
    }
    roots.sort();
    roots.dedup();
    roots
}

fn swift_dependency_targets(
    facts: &crate::codebase::swift::SwiftFactMap,
    source_package: &crate::codebase::swift::SwiftPackageFacts,
    target: &crate::codebase::swift::SwiftTargetFacts,
    dependency: &str,
) -> Vec<(PathBuf, Vec<String>)> {
    swift_dependency_packages(facts, source_package, target, dependency)
        .into_iter()
        .filter_map(|package| {
            let names = package.products.get(dependency).cloned().or_else(|| {
                package
                    .targets
                    .contains_key(dependency)
                    .then(|| vec![dependency.to_string()])
            })?;
            Some((package.package_root.clone(), names))
        })
        .collect()
}

fn swift_dependency_packages<'a>(
    facts: &'a crate::codebase::swift::SwiftFactMap,
    source_package: &'a crate::codebase::swift::SwiftPackageFacts,
    target: &crate::codebase::swift::SwiftTargetFacts,
    dependency: &str,
) -> Vec<&'a crate::codebase::swift::SwiftPackageFacts> {
    if source_package.targets.contains_key(dependency) {
        return vec![source_package];
    }
    let explicit_identity = target.product_packages.get(dependency);
    source_package
        .local_package_bindings
        .iter()
        .filter(|(_, identity)| explicit_identity.is_none_or(|expected| expected == *identity))
        .filter_map(|(local, _)| {
            let root = crate::codebase::ts_resolver::normalize_path(
                &source_package.package_root.join(local),
            );
            facts.packages.iter().find(|package| package.package_root == root)
        })
        .filter(|package| {
            explicit_identity.is_some()
                || package.products.contains_key(dependency)
                || package.targets.contains_key(dependency)
        })
        .collect()
}

fn swift_owning_package<'a>(
    facts: &'a crate::codebase::swift::SwiftFactMap,
    path: &Path,
) -> Option<&'a crate::codebase::swift::SwiftPackageFacts> {
    facts
        .packages
        .iter()
        .filter(|package| path.starts_with(&package.package_root))
        .max_by_key(|package| package.package_root.components().count())
}
