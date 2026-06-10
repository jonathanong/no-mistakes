use crate::check_tasks::{
    run_codebase_check, run_filesystem_rules_check, run_integration_check, run_queue_check,
    run_react_check, run_rules_check, CheckTask,
};
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::path::{Path, PathBuf};

pub(crate) type DomainResults = (
    anyhow::Result<CheckTask<Vec<react_traits::Violation>>>,
    anyhow::Result<CheckTask<Vec<CheckFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
    anyhow::Result<CheckTask<Vec<IntegrationFinding>>>,
    anyhow::Result<CheckTask<Vec<UniqueExportFinding>>>,
    anyhow::Result<CheckTask<Vec<RuleFinding>>>,
);

pub(crate) struct DomainCheckInputs<'a> {
    pub(crate) root: &'a Path,
    pub(crate) config_path: &'a Option<PathBuf>,
    pub(crate) tsconfig_path: &'a Option<PathBuf>,
    pub(crate) react_enabled: bool,
    pub(crate) queues_enabled: bool,
    pub(crate) unique_exports_enabled: bool,
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) discovered_files: Vec<PathBuf>,
    pub(crate) facts: &'a CheckFactMap,
}

pub(crate) fn run_domain_checks(inputs: DomainCheckInputs<'_>) -> DomainResults {
    let root = inputs.root;
    let config_path = inputs.config_path;
    let tsconfig_path = inputs.tsconfig_path;
    let react_enabled = inputs.react_enabled;
    let queues_enabled = inputs.queues_enabled;
    let unique_exports_enabled = inputs.unique_exports_enabled;
    let filesystem_rules_enabled = inputs.filesystem_rules_enabled;
    let discovered_files = inputs.discovered_files;
    let facts = inputs.facts;

    let ((react, queues), (rules, (integration, (codebase, filesystem_rules)))) = rayon::join(
        || {
            rayon::join(
                || run_react_check(root, config_path.as_deref(), react_enabled, facts),
                || run_queue_check(root, tsconfig_path.as_deref(), queues_enabled, facts),
            )
        },
        || {
            rayon::join(
                || {
                    run_rules_check(
                        root,
                        config_path.as_deref(),
                        tsconfig_path.as_deref(),
                        facts,
                    )
                },
                || {
                    rayon::join(
                        || run_integration_check(root, config_path.as_deref(), facts),
                        || {
                            rayon::join(
                                || {
                                    run_codebase_check(
                                        root,
                                        config_path.as_deref(),
                                        tsconfig_path.as_deref(),
                                        unique_exports_enabled,
                                        facts,
                                    )
                                },
                                || {
                                    run_filesystem_rules_check(
                                        root,
                                        config_path.as_deref(),
                                        filesystem_rules_enabled,
                                        &discovered_files,
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    );
    (
        react,
        queues,
        rules,
        integration,
        codebase,
        filesystem_rules,
    )
}
