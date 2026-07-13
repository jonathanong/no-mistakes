use super::super::frameworks::framework_present;
use super::super::ImpactedChecksArgs;
use crate::config::v2::schema::NoMistakesConfig;
use crate::tests::prepared_plan::{PreparedTestPlanInputs, PreparedTestPlanRequest};
use crate::tests::TestFramework;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) struct PreparedImpactedChecks {
    pub(super) frameworks: Vec<TestFramework>,
    pub(super) changed_files: Vec<String>,
    pub(super) missing: BTreeSet<String>,
    request: PreparedRequest,
}

enum PreparedRequest {
    GenericOnly(Box<PreparedTestPlanInputs>),
    Frameworks(Box<PreparedTestPlanRequest>),
}

impl PreparedImpactedChecks {
    pub(super) fn config(&self) -> &NoMistakesConfig {
        match &self.request {
            PreparedRequest::GenericOnly(inputs) => &inputs.config,
            PreparedRequest::Frameworks(prepared) => &prepared.config,
        }
    }

    pub(super) fn framework_request(&self) -> Option<&PreparedTestPlanRequest> {
        match &self.request {
            PreparedRequest::GenericOnly(_) => None,
            PreparedRequest::Frameworks(prepared) => Some(prepared),
        }
    }

    pub(super) fn stats(&self) -> super::PlanStats {
        match &self.request {
            PreparedRequest::GenericOnly(_) => super::PlanStats {
                framework_discoveries: 0,
                graph_builds: 0,
            },
            PreparedRequest::Frameworks(prepared) => super::PlanStats {
                framework_discoveries: prepared.framework_discovery_count(),
                graph_builds: prepared.graph_build_count(),
            },
        }
    }
}

pub(super) fn prepare_impacted_checks(args: &ImpactedChecksArgs) -> Result<PreparedImpactedChecks> {
    let plan_args = super::args::plan_args_for(args, None);
    let inputs = PreparedTestPlanInputs::prepare(&plan_args)?;
    let frameworks =
        configured_frameworks(&inputs.root, &inputs.config, inputs.root_visible_paths());
    let changed_files = sorted_unique(
        inputs
            .collected
            .files
            .iter()
            .map(|file| relative_slash(&inputs.root, file)),
    );
    // Append-style checks must skip files that no longer exist, while
    // whole-project checks still trigger for them.
    let missing = inputs
        .collected
        .files
        .iter()
        .filter(|file| !file.exists())
        .map(|file| relative_slash(&inputs.root, file))
        .collect();
    let request = if frameworks.is_empty() {
        PreparedRequest::GenericOnly(Box::new(inputs))
    } else {
        PreparedRequest::Frameworks(Box::new(inputs.finish()?))
    };

    Ok(PreparedImpactedChecks {
        frameworks,
        changed_files,
        missing,
        request,
    })
}

fn configured_frameworks(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<TestFramework> {
    [
        TestFramework::Dotnet,
        TestFramework::Vitest,
        TestFramework::Playwright,
        TestFramework::Swift,
    ]
    .into_iter()
    .filter(|framework| framework_present(root, config, *framework, visible_paths))
    .collect()
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    let set: BTreeSet<String> = values.collect();
    set.into_iter().collect()
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
