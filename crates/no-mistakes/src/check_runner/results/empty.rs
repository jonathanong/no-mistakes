use super::CheckResults;
use crate::check_tasks::CheckTask;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;

pub(crate) struct CompletedDomainChecks {
    pub(crate) react: CheckTask<Vec<react_traits::Violation>>,
    pub(crate) queues: CheckTask<Vec<CheckFinding>>,
    pub(crate) rules: CheckTask<Vec<RuleFinding>>,
    pub(crate) integration: CheckTask<Vec<IntegrationFinding>>,
    pub(crate) codebase: CheckTask<Vec<PreparedUniqueExportFinding>>,
    pub(crate) filesystem_rules: CheckTask<Vec<RuleFinding>>,
}

pub(super) fn timings(d: [Duration; 8]) -> Vec<(&'static str, Duration)> {
    const NAMES: [&str; 8] = [
        "discover",
        "parse_extract",
        "react",
        "queues",
        "rules",
        "integration",
        "codebase",
        "filesystem_rules",
    ];
    NAMES.into_iter().zip(d).collect()
}

pub(crate) fn empty_results(warnings: [Option<String>; 1]) -> CheckResults {
    let warnings = warnings.into_iter().flatten().collect();
    CheckResults {
        incomplete: Vec::new(),
        react: Vec::new(),
        queues: Vec::new(),
        rules: Vec::new(),
        integration: Vec::new(),
        runner_config_deadlines: None,
        codebase: Vec::new(),
        warnings,
        analysis_warning_count: 0,
        analysis_fallbacks: Vec::new(),
        advisories: Vec::new(),
        sql_verdicts: Vec::new(),
        include_sql_verdicts: false,
        suppressed: Vec::new(),
        include_suppressed: false,
        unproven_member_query_sql: 0,
        accepted_bounds: Vec::new(),
        timings: timings([Duration::ZERO; 8]),
    }
}
