use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

use super::types::TestRunner;

#[derive(Clone)]
pub struct ProjectTestFilter {
    include: GlobSet,
    exclude: Option<GlobSet>,
}

impl ProjectTestFilter {
    pub(super) fn from_project(project: ConfigProject) -> Option<Self> {
        Self::from_project_ref(&project).ok()
    }

    pub(super) fn from_project_ref(project: &ConfigProject) -> Result<Self> {
        Ok(Self {
            include: compile_globset(&project.include)?,
            exclude: compile_optional_globset(&project.exclude)?,
        })
    }

    pub fn is_match(&self, rel_path: &str) -> bool {
        self.include.is_match(rel_path)
            && self
                .exclude
                .as_ref()
                .is_none_or(|exclude| !exclude.is_match(rel_path))
    }

    pub fn includes(&self, rel_path: &str) -> bool {
        self.include.is_match(rel_path)
    }

    pub fn excludes(&self, rel_path: &str) -> bool {
        self.exclude
            .as_ref()
            .is_some_and(|exclude| exclude.is_match(rel_path))
    }
}

pub fn fallback_test_path(rel_path: &str) -> bool {
    rel_path
        .split('/')
        .any(|component| component == "__tests__")
        || rel_path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.contains(".test.") || name.contains(".spec."))
}

pub(super) fn fallback_runner_match(runner: TestRunner, rel: &str) -> bool {
    match runner {
        TestRunner::Vitest => {
            fallback_test_path(rel)
                && !rel.split('/').any(|component| component == "playwright")
                && !has_path_segment_pair(rel, "tests", "e2e")
                && !rel.starts_with("specs/")
        }
        TestRunner::Playwright => {
            rel.contains("/tests/e2e/")
                || rel.starts_with("tests/e2e/")
                || rel.contains("/playwright/")
                || rel.starts_with("playwright/")
                || rel.starts_with("specs/")
        }
    }
}

fn has_path_segment_pair(path: &str, first: &str, second: &str) -> bool {
    let mut segments = path.split('/');
    let Some(mut previous) = segments.next() else {
        return false;
    };
    for current in segments {
        if previous == first && current == second {
            return true;
        }
        previous = current;
    }
    false
}

fn compile_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn compile_optional_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    compile_globset(patterns).map(Some)
}
