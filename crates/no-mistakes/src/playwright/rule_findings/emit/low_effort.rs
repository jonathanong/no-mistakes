use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::Unanalyzable;
use crate::playwright::analysis::types::CoverageReport;
use crate::playwright::rules::PLAYWRIGHT_UNIQUE_HTML_IDS;
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;

pub(in crate::playwright::rule_findings) fn unanalyzable(
    diagnostics: &mut AnalysisDiagnostics,
    report: &CoverageReport,
    rule: &str,
    policy: Option<Unanalyzable>,
    skip_html_ids: bool,
) {
    let Some(policy) = policy else {
        return;
    };
    for selector in &report.selectors {
        if selector.literal_exact {
            continue;
        }
        if skip_html_ids && selector.attribute == HTML_ID_ATTRIBUTE {
            continue;
        }
        diagnostics.unanalyzable(
            policy,
            RuleFinding {
                rule: rule.to_string(),
                file: selector.file.clone(),
                line: selector.line,
                message: format!(
                    "selector [{}=\"{}\"] is unanalyzable at effort low; {}",
                    selector.attribute,
                    selector.value,
                    crate::playwright::rules::effort::REWRITE
                ),
                import: None,
                target: Some(format!("{}={}", selector.attribute, selector.value)),
            },
        );
    }
}

pub(in crate::playwright::rule_findings) fn unanalyzable_html_ids(
    diagnostics: &mut AnalysisDiagnostics,
    selectors: &[crate::playwright::selectors::AppSelector],
    policy: Option<Unanalyzable>,
) {
    let Some(policy) = policy else {
        return;
    };
    for selector in selectors {
        let value = selector.display_value();
        diagnostics.unanalyzable(
            policy,
            RuleFinding {
                rule: PLAYWRIGHT_UNIQUE_HTML_IDS.to_string(),
                file: selector.file.to_string_lossy().into_owned(),
                line: selector.line,
                message: format!(
                    "selector [id=\"{value}\"] is unanalyzable at effort low; {}",
                    crate::playwright::rules::effort::HTML_ID_REWRITE,
                ),
                import: None,
                target: Some(format!("id={value}")),
            },
        );
    }
}
