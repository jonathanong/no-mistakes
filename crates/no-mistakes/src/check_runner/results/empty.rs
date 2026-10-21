use super::CheckResults;
use std::time::Duration;

pub(crate) fn empty_results(warnings: [Option<String>; 1]) -> CheckResults {
    let warnings = warnings.into_iter().flatten().collect();
    CheckResults {
        react: Vec::new(),
        queues: Vec::new(),
        rules: Vec::new(),
        integration: Vec::new(),
        runner_config_deadlines: None,
        codebase: Vec::new(),
        warnings,
        advisories: Vec::new(),
        sql_verdicts: Vec::new(),
        include_sql_verdicts: false,
        suppressed: Vec::new(),
        include_suppressed: false,
        unproven_member_query_sql: 0,
        timings: vec![
            ("discover", Duration::ZERO),
            ("parse_extract", Duration::ZERO),
            ("react", Duration::ZERO),
            ("queues", Duration::ZERO),
            ("rules", Duration::ZERO),
            ("integration", Duration::ZERO),
            ("codebase", Duration::ZERO),
            ("filesystem_rules", Duration::ZERO),
        ],
    }
}
