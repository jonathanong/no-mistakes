use super::TestProjectPolicy;
use anyhow::Result;
use std::path::{Component, Path};

impl TestProjectPolicy {
    pub(crate) fn validate_vitest_setup_files(&self, name: &str) -> Result<()> {
        anyhow::ensure!(
            self.setup_files.is_empty() || !self.include.is_empty(),
            "tests.vitest.projects.{name}.setup_files requires a nonempty include matcher"
        );
        for setup in &self.setup_files {
            let relative = Path::new(setup);
            anyhow::ensure!(
                !setup.is_empty()
                    && !relative.is_absolute()
                    && relative.components().all(|part| matches!(
                        part,
                        Component::Normal(_) | Component::CurDir
                    )),
                "tests.vitest.projects.{name}.setup_files contains invalid repository-relative path {setup:?}"
            );
        }
        Ok(())
    }
}
