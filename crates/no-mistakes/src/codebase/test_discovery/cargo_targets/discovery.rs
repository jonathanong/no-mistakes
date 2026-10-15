use super::*;

impl CargoTargets {
    fn owner(&self, file: &Path) -> Option<(&Path, &Package)> {
        let directory = file
            .ancestors()
            .skip(1)
            .find(|directory| self.manifests.contains(*directory))?;
        self.packages
            .get_key_value(directory)
            .map(|(directory, package)| (directory.as_path(), package))
    }
    fn owners<'a>(
        &'a self,
        file: &'a Path,
    ) -> impl Iterator<Item = (&'a Path, &'a Package)> + Clone {
        self.owner(file).into_iter().chain(
            self.external_entries
                .get(file)
                .into_iter()
                .flatten()
                .filter_map(|directory| {
                    self.packages
                        .get_key_value(directory)
                        .map(|(directory, package)| (directory.as_path(), package))
                }),
        )
    }
    pub(in crate::codebase::test_discovery) fn discover(
        &self,
        root: &Path,
        files: &[PathBuf],
    ) -> DiscoveredTests {
        let mut tests = Vec::new();
        let mut targets_by_path = BTreeMap::new();
        for file in files {
            if !self.scopes.is_empty()
                && !self.external_entries.contains_key(file)
                && !file
                    .ancestors()
                    .skip(1)
                    .find(|directory| self.manifests.contains(*directory))
                    .is_some_and(|directory| self.scopes.contains(directory))
            {
                continue;
            }
            let relative = crate::codebase::ts_source::relative_slash_path(root, file);
            let owners = self.owners(file);
            let explicit_test = owners
                .clone()
                .any(|(_, package)| package.is_test_entry(file));
            if !explicit_test
                && !super::super::filters::fallback_runner_match(
                    super::super::TestRunner::Cargo,
                    &relative,
                )
            {
                continue;
            }
            tests.push(file.clone());
            let targets = owners
                .flat_map(|(directory, package)| self.targets(root, directory, package, file))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            targets_by_path.insert(file.clone(), targets);
        }
        tests.sort();
        tests.dedup();
        DiscoveredTests {
            tests,
            targets_by_path,
            used_fallback: false,
        }
    }
}
