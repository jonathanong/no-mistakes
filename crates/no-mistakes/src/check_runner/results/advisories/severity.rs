//! `severity: advisory` moves a rule's findings to the non-failing channel.
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::config::v2::{DisableDirectivePolicy, NoMistakesConfig};

/// Runs after suppression so disable directives still apply first. Findings are
/// moved only when every enabled application of the rule is advisory. Allow
/// hygiene follows its own policy after the rule scope has proved staleness.
pub(in crate::check_runner::results) fn demote(
    config: &NoMistakesConfig,
    queues: &mut Vec<no_mistakes::queue::CheckFinding>,
    rules: &mut Vec<RuleFinding>,
    codebase: &mut Vec<PreparedUniqueExportFinding>,
    advisories: &mut Vec<RuleFinding>,
) {
    let (advisory_queues, failing_queues): (Vec<_>, Vec<_>) = std::mem::take(queues)
        .into_iter()
        .partition(no_mistakes::queue::CheckFinding::is_advisory);
    *queues = failing_queues;
    advisories.extend(advisory_queues.into_iter().map(|finding| RuleFinding {
        causes: Vec::new(),
        rule: "queues-check".to_string(),
        file: finding.file,
        line: finding.line,
        message: finding.message,
        import: None,
        target: None,
    }));
    rules.retain(|finding| {
        finding.rule != "unused-allow-entry"
            || config.report_unused_allow_entries != DisableDirectivePolicy::Ignore
    });
    let (advisory, failing) = std::mem::take(rules).into_iter().partition(|finding| {
        if finding.rule == "unused-allow-entry" {
            config.report_unused_allow_entries == DisableDirectivePolicy::Advise
        } else {
            config.rule_is_advisory(&finding.rule)
        }
    });
    *rules = failing;
    advisories.extend(advisory);
    let (advisory, failing): (Vec<_>, Vec<_>) = std::mem::take(codebase)
        .into_iter()
        .partition(|prepared| config.rule_is_advisory(&prepared.finding.rule));
    *codebase = failing;
    advisories.extend(advisory.into_iter().map(|prepared| {
        let finding = prepared.finding;
        RuleFinding {
            causes: Vec::new(),
            rule: finding.rule,
            file: finding.file,
            line: finding.line as usize,
            message: finding.message,
            import: None,
            target: None,
        }
    }));
}
