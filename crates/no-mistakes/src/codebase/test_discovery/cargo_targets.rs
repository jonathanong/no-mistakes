use super::{DiscoveredTests, NoMistakesConfig, TestExecutionTarget};
use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod discovery;
mod manifest;
mod manifests;
mod targets;
use targets::Package;

/// A request-owned Cargo projection; it borrows the prepared inventory and
/// source identity instead of invoking Cargo or discovering another file tree.
#[derive(Default)]
pub(super) struct CargoTargets {
    packages: BTreeMap<PathBuf, Package>,
    manifests: BTreeSet<PathBuf>,
    external_entries: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    scopes: BTreeSet<PathBuf>,
    #[cfg(any(test, feature = "test-instrumentation"))]
    membership_index_builds: usize,
}
impl CargoTargets {
    pub(super) fn prepare(
        root: &Path,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> Self {
        let mut result = Self {
            scopes: config
                .tests
                .rust
                .packages
                .iter()
                .map(|scope| crate::codebase::ts_resolver::normalize_path(&root.join(scope)))
                .collect(),
            ..Self::default()
        };
        let paths: Vec<_> = files
            .iter()
            .filter(|path| path.file_name().is_some_and(|name| name == "Cargo.toml"))
            .collect();
        if paths.is_empty() {
            return result;
        }
        let visible: BTreeSet<_> = files.iter().map(PathBuf::as_path).collect();
        #[cfg(any(test, feature = "test-instrumentation"))]
        {
            result.membership_index_builds += 1;
        }
        let mut manifests = manifests::Manifests::new(sources);
        for path in paths {
            let directory = path.parent().expect("manifest filenames have a parent");
            result.manifests.insert(directory.to_path_buf());
            if !result.scopes.is_empty() && !result.scopes.contains(directory) {
                continue;
            }
            let edition = manifests.edition(path, &visible);
            let Some(value) = manifests.read(path) else {
                continue;
            };
            if let Some(package) = manifest::parse(directory, value, &visible, edition.as_deref()) {
                for target in &package.targets {
                    if !target.entry.starts_with(directory) {
                        result
                            .external_entries
                            .entry(target.entry.clone())
                            .or_default()
                            .insert(directory.to_path_buf());
                    }
                }
                result.packages.insert(directory.to_path_buf(), package);
            }
        }
        result
    }
    pub(super) fn projects(
        &self,
        root: &Path,
    ) -> Vec<crate::integration_tests::types::ConfigProject> {
        self.packages
            .iter()
            // Package-wide source fallback is an explicit configuration opt-in.
            .filter(|(directory, _)| self.scopes.contains(*directory))
            .map(|(directory, package)| {
                let scope = package_scope(root, directory);
                let mut include = crate::integration_tests::project_config::prefix_globs(
                    root,
                    directory,
                    &crate::codebase::dependencies::test_globs("cargo"),
                );
                include.extend(
                    package
                        .targets
                        .iter()
                        .filter(|target| target.flag == "--test" && target.enabled)
                        .map(|target| {
                            super::literal_path_glob(
                                &crate::codebase::ts_source::relative_slash_path(
                                    root,
                                    &target.entry,
                                ),
                            )
                        }),
                );
                crate::integration_tests::types::ConfigProject {
                    config: Some(scope.clone()),
                    workspace: false,
                    policy_name: Some(scope.clone()),
                    runner_project_arg: Some(package.name.clone()),
                    scope: Some(scope),
                    include,
                    exclude: Vec::new(),
                    declared_deadlines: Default::default(),
                    matcher_warnings: Vec::new(),
                    vitest_setup: Vec::new(),
                }
            })
            .collect()
    }
    fn targets(
        &self,
        root: &Path,
        directory: &Path,
        package: &Package,
        file: &Path,
    ) -> Vec<TestExecutionTarget> {
        let candidates = package.target_ids(file);
        let manifest =
            crate::codebase::ts_source::relative_slash_path(root, &directory.join("Cargo.toml"));
        let scope = package_scope(root, directory);
        candidates
            .iter()
            .map(|id| &package.targets[*id])
            .filter(|target| target.enabled)
            .map(|target| {
                let mut args = vec![
                    "--manifest-path".into(),
                    manifest.clone(),
                    "-p".into(),
                    package.name.clone(),
                    target.flag.into(),
                ];
                if target.flag != "--lib" {
                    args.push(target.name.clone());
                }
                TestExecutionTarget {
                    runner: "cargo".into(),
                    config: Some(scope.clone()),
                    workspace: false,
                    project: Some(package.name.clone()),
                    base_command: vec!["cargo".into(), "test".into()],
                    runner_args: args,
                }
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

fn package_scope(root: &Path, directory: &Path) -> String {
    let scope = crate::codebase::ts_source::relative_slash_path(root, directory);
    if scope.is_empty() {
        ".".into()
    } else {
        scope
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod external_tests;
