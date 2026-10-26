use super::{marked_lines, read, run, RULE};
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::config::v2::Effort;
use std::path::PathBuf;

const HTML_RULE: &str = "playwright-unique-html-ids";

fn fixture() -> PathBuf {
    super::effort_root()
        .parent()
        .unwrap()
        .join("html-id-ownership")
}

fn shape_lines(rows: &[RuleFinding], file: &str, rule: &str, attribute: &str) -> Vec<usize> {
    let mut lines: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.file == file
                && row.rule == rule
                && row.message.contains("unanalyzable at effort low")
                && row
                    .target
                    .as_deref()
                    .is_some_and(|target| target.starts_with(attribute))
        })
        .map(|row| row.line)
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn html_coverage_does_not_add_test_id_uncertainty_under_any_policy() {
    let root = fixture();
    for policy in ["report", "advise", "ignore"] {
        let result = run(&root, Some(&format!("coverage-{policy}.yml")), None);
        let rows: Vec<_> = result.rules.iter().chain(&result.advisories).collect();
        assert!(
            rows.iter().all(|row| {
                row.rule != RULE
                    || !row
                        .target
                        .as_deref()
                        .is_some_and(|target| target.starts_with("id="))
            }),
            "{policy}: {rows:#?}"
        );
        let shapes = if policy == "report" {
            &result.rules
        } else {
            &result.advisories
        };
        let page = "web/app/page.tsx";
        assert_eq!(
            shape_lines(shapes, page, RULE, "data-testid="),
            if policy == "ignore" {
                vec![]
            } else {
                marked_lines(&read(&root, page), "EXPECT_TEST")
            },
            "{policy}"
        );
        // Literal test-ID duplicates remain definite failures for every policy.
        assert_eq!(
            super::duplicate_targets(&result.rules),
            ["data-testid=literal-test"; 2]
        );
        assert!(
            rows.iter().all(|row| !row.file.contains("suppressed")),
            "{rows:#?}"
        );
    }
}

#[test]
fn explicitly_configured_id_hooks_keep_their_low_effort_diagnostics() {
    let root = fixture();
    for policy in ["report", "advise", "ignore"] {
        let result = run(&root, Some(&format!("configured-{policy}.yml")), None);
        let shapes = if policy == "report" {
            &result.rules
        } else {
            &result.advisories
        };
        for file in ["web/app/page.tsx", "web/app/other.tsx"] {
            assert_eq!(
                shape_lines(shapes, file, RULE, "id="),
                if policy == "ignore" {
                    vec![]
                } else {
                    marked_lines(&read(&root, file), "EXPECT_HTML")
                },
                "{policy}: {file}: {shapes:#?}"
            );
        }
        assert!(result
            .rules
            .iter()
            .chain(&result.advisories)
            .all(|row| !row.file.contains("suppressed")));
    }
}

#[test]
fn component_id_mappings_and_dedicated_html_rule_retain_ownership() {
    let root = fixture();
    for (config, rule) in [
        ("component-report.yml", RULE),
        ("dedicated-html-report.yml", HTML_RULE),
    ] {
        let result = run(&root, Some(config), None);
        let files = if config == "component-report.yml" {
            vec!["web/app/component.tsx"]
        } else {
            vec!["web/app/page.tsx", "web/app/other.tsx"]
        };
        let marker = if config == "component-report.yml" {
            "EXPECT_COMPONENT"
        } else {
            "EXPECT_HTML"
        };
        for file in files {
            assert_eq!(
                shape_lines(&result.rules, file, rule, "id="),
                marked_lines(&read(&root, file), marker),
                "{config}: {:?}",
                result.rules
            );
        }
        if rule == HTML_RULE {
            assert!(result.rules.iter().all(|row| {
                row.rule != RULE
                    || !row
                        .target
                        .as_deref()
                        .is_some_and(|target| target.starts_with("id="))
            }));
        }
        assert!(result
            .rules
            .iter()
            .chain(&result.advisories)
            .all(|row| !row.file.contains("suppressed")));
    }
}

#[test]
fn html_coverage_only_max_reports_remain_identical() {
    let root = fixture();
    let default = run(&root, Some("coverage-default.yml"), None);
    let explicit = run(&root, Some("coverage-max.yml"), None);
    let forced = run(&root, Some("coverage-default.yml"), Some(Effort::Max));
    assert_eq!(
        crate::check_runner::json_value(&default),
        crate::check_runner::json_value(&explicit)
    );
    assert_eq!(
        crate::check_runner::json_value(&default),
        crate::check_runner::json_value(&forced)
    );
    assert_eq!(
        super::duplicate_targets(&default.rules),
        ["data-testid=literal-test"; 2]
    );
    assert!(default
        .rules
        .iter()
        .chain(&default.advisories)
        .all(|row| !row.message.contains("unanalyzable at effort low")));
}

#[test]
fn standalone_certified_low_policy_respects_html_id_ownership_and_findings_only_contract() {
    use no_mistakes::config::v2::{EffectiveRuleAnalysis, Unanalyzable};

    let root = fixture();
    for (file, policy, html_shapes) in [
        ("coverage-report.yml", Unanalyzable::Report, 0),
        ("configured-report.yml", Unanalyzable::Report, 4),
        ("component-report.yml", Unanalyzable::Report, 1),
        ("dedicated-html-report.yml", Unanalyzable::Report, 0),
        ("coverage-advise.yml", Unanalyzable::Advise, 0),
    ] {
        let path = root.join(file);
        let mut config = no_mistakes::config::v2::load_v2_config(&root, Some(&path)).unwrap();
        for application in &mut config.rules {
            // This lower-level adapter consumes policies certified by its caller.
            application.resolved_analysis = Some(EffectiveRuleAnalysis {
                effort: Effort::Low,
                unanalyzable: policy,
            });
        }
        let rows = no_mistakes::playwright::rules::check(&root, Some(&path), &config).unwrap();
        assert_eq!(
            rows.iter()
                .filter(|row| row.rule == RULE
                    && row.message.contains("unanalyzable")
                    && row
                        .target
                        .as_deref()
                        .is_some_and(|target| target.starts_with("id=")))
                .count(),
            html_shapes,
            "{file}: {rows:#?}"
        );
        if policy == Unanalyzable::Advise {
            assert_eq!(rows.len(), 2, "{rows:#?}");
            assert!(rows.iter().all(|row| row.message.contains("duplicated")));
            assert_eq!(run(&root, Some(file), None).advisories.len(), 1);
        }
    }
}
