use super::super::RuleFinding;
use super::{
    coverage_paths::CoveragePath, projects::CoverageUnit, workflow_filters::CiFilter, RULE_ID,
};
use std::collections::BTreeSet;

pub(super) fn missed_path(
    filters: &[&CiFilter],
    unit: &CoverageUnit,
    path: CoveragePath,
) -> RuleFinding {
    let filter_list = filters
        .iter()
        .map(|filter| format!("{}:{}", filter.workflow, filter.name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: filters[0].workflow.clone(),
        line: 1,
        message: format!(
            "{}: Vitest project `{}` {}{} is not covered by CI path filters: {filter_list}",
            path.rel,
            unit.project,
            unit.source.label(),
            if path.synthetic {
                " glob witness path"
            } else {
                " path"
            }
        ),
        import: None,
        target: Some(path.rel),
    }
}
