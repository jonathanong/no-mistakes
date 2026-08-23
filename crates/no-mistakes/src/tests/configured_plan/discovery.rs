use super::{configured_environment, PlanArgs, TestFramework};
use crate::tests::plan::relative_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use std::collections::HashSet;
use std::path::PathBuf;

pub(crate) fn discover_framework_tests_from_prepared(
    args: &PlanArgs,
    framework: TestFramework,
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
) -> Result<DiscoveredTests> {
    let env = configured_environment(args, framework, &prepared.config)?;
    let mut discovered = prepared.discover_tests(framework)?;
    let include = compile_globset(&env.include)?;
    let exclude = compile_globset(&env.exclude)?;
    discovered.tests.retain(|path| {
        let rel = relative_path(&prepared.root, path);
        include.as_ref().is_none_or(|set| set.is_match(&rel))
            && exclude.as_ref().is_none_or(|set| !set.is_match(&rel))
    });
    let allowed: HashSet<PathBuf> = discovered.tests.iter().cloned().collect();
    discovered
        .targets_by_path
        .retain(|path, _| allowed.contains(path));
    Ok(discovered)
}

fn compile_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}
