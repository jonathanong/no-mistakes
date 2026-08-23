use super::TestPlan;
use crate::tests::{ImpactEdgeDetail, ImpactReason};
use anyhow::Result;
use std::fmt::Write;

pub(super) fn render(plan: &TestPlan, output: &mut String) -> Result<()> {
    writeln!(
        output,
        "Test plan: {} selected test(s)",
        plan.selected_tests.len()
    )?;
    if plan.fallback_triggered {
        writeln!(output, "Fallback: triggered")?;
        if let Some(reason) = &plan.fallback_reason {
            writeln!(output, "Fallback reason: {reason}")?;
        }
    } else {
        writeln!(output, "Fallback: not triggered")?;
    }

    let mut changed_files = plan.changed_files.iter().collect::<Vec<_>>();
    changed_files.sort();
    changed_files.dedup();
    writeln!(output, "\nChanged files ({}):", changed_files.len())?;
    if changed_files.is_empty() {
        writeln!(output, "- None")?;
    } else {
        for changed_file in changed_files {
            writeln!(output, "- {changed_file}")?;
        }
    }

    let mut selected = plan.selected_tests.iter().collect::<Vec<_>>();
    selected.sort_by(|left, right| left.test_file.cmp(&right.test_file));
    if selected.is_empty() {
        writeln!(output, "\nNo tests selected.")?;
    }
    for test in selected {
        writeln!(output, "\nTest: {}", test.test_file)?;
        writeln!(output, "Confidence: {}", test.confidence.display_emoji())?;
        let mut reasons = test.reasons.iter().collect::<Vec<_>>();
        reasons.sort_by(|left, right| {
            (&left.changed_file, &left.path, &left.via)
                .cmp(&(&right.changed_file, &right.path, &right.via))
                .then_with(|| {
                    format!("{:?}", left.via_details).cmp(&format!("{:?}", right.via_details))
                })
        });
        for reason in reasons {
            writeln!(output, "Reason: {}", reason.changed_file)?;
            writeln!(output, "  Path: {}", path(reason))?;
        }
    }

    let mut warnings = plan.warnings.iter().collect::<Vec<_>>();
    warnings.sort_by(|left, right| {
        (&left.file, left.line, &left.r#type, &left.message).cmp(&(
            &right.file,
            right.line,
            &right.r#type,
            &right.message,
        ))
    });
    if !warnings.is_empty() {
        writeln!(output, "\nWarnings ({}):", warnings.len())?;
        for warning in warnings {
            let location = warning.line.map_or_else(
                || warning.file.clone(),
                |line| format!("{}:{line}", warning.file),
            );
            writeln!(
                output,
                "- {} ({location}): {}",
                warning.r#type, warning.message
            )?;
        }
    }
    Ok(())
}

fn path(reason: &ImpactReason) -> String {
    if reason.path.len() == 1 && reason.via.iter().any(|via| via == "self") {
        return format!("`{}` (self-selected)", reason.path[0]);
    }

    let mut rendered = Vec::with_capacity(reason.path.len().saturating_mul(2));
    for (index, node) in reason.path.iter().enumerate() {
        rendered.push(format!("`{node}`"));
        // Edge labels belong between adjacent path nodes. A self-selection
        // reason has one node and is rendered above as node provenance rather
        // than as an edge leading nowhere.
        if index + 1 < reason.path.len() {
            if let Some(via) = reason.via.get(index) {
                rendered.push(format!(
                    "[{}]",
                    display_via(via, reason.via_details.get(index).and_then(Option::as_ref))
                ));
            }
        }
    }
    rendered.join(" ➔ ")
}

fn display_via(via: &str, detail: Option<&ImpactEdgeDetail>) -> String {
    match detail {
        Some(ImpactEdgeDetail::VitestSetup { field }) => format!("{via} ({field})"),
        Some(ImpactEdgeDetail::Resource {
            consumer_file,
            call_sites,
        }) if !call_sites.is_empty() => {
            format!(
                "{via} ({consumer_file}: {})",
                call_sites
                    .iter()
                    .map(|site| format!("{} line {}", site.call_kind, site.line))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        Some(ImpactEdgeDetail::Resource { consumer_file, .. }) => {
            format!("{via} ({consumer_file})")
        }
        None => via.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{Confidence, PlanFormat, ResourceCallSite, SelectedTest, Warning};

    #[test]
    fn explain_is_deterministic_and_renders_provenance() {
        let plan = TestPlan {
            changed_files: vec!["z.ts".to_string(), "a.ts".to_string()],
            selected_tests: vec![
                selected(
                    "z.test.ts",
                    Confidence::Medium,
                    "z.ts",
                    "vitest-setup",
                    Some(ImpactEdgeDetail::VitestSetup {
                        field: "setupFiles".to_string(),
                    }),
                ),
                selected(
                    "a.test.ts",
                    Confidence::High,
                    "a.ts",
                    "resource",
                    Some(ImpactEdgeDetail::Resource {
                        consumer_file: "a.test.ts".to_string(),
                        call_sites: vec![ResourceCallSite {
                            call_kind: "read-file".to_string(),
                            line: 4,
                        }],
                    }),
                ),
            ],
            groups: Vec::new(),
            warnings: vec![Warning {
                r#type: "dynamic-import".to_string(),
                message: "might not resolve".to_string(),
                file: "a.ts".to_string(),
                line: Some(3),
            }],
            fallback_triggered: false,
            fallback_reason: None,
            ..Default::default()
        };

        assert_eq!(
            super::super::render(&plan, PlanFormat::Explain, "tests plan").unwrap(),
            "Test plan: 2 selected test(s)\nFallback: not triggered\n\nChanged files (2):\n- a.ts\n- z.ts\n\nTest: a.test.ts\nConfidence: 🟢 High\nReason: a.ts\n  Path: `a.ts` ➔ [resource (a.test.ts: read-file line 4)] ➔ `a.test.ts`\n\nTest: z.test.ts\nConfidence: 🟡 Medium\nReason: z.ts\n  Path: `z.ts` ➔ [vitest-setup (setupFiles)] ➔ `z.test.ts`\n\nWarnings (1):\n- dynamic-import (a.ts:3): might not resolve\n"
        );
    }

    #[test]
    fn explain_renders_self_selection_without_a_dangling_edge() {
        let plan = TestPlan {
            changed_files: vec!["src/unit.test.ts".to_string()],
            selected_tests: vec![SelectedTest {
                test_file: "src/unit.test.ts".to_string(),
                confidence: Confidence::High,
                targets: Vec::new(),
                reasons: vec![ImpactReason {
                    changed_file: "src/unit.test.ts".to_string(),
                    path: vec!["src/unit.test.ts".to_string()],
                    via: vec!["self".to_string()],
                    via_details: Vec::new(),
                }],
            }],
            groups: Vec::new(),
            warnings: Vec::new(),
            fallback_triggered: false,
            fallback_reason: None,
            ..Default::default()
        };

        let rendered = super::super::render(&plan, PlanFormat::Explain, "tests plan").unwrap();
        assert!(rendered.contains("Path: `src/unit.test.ts` (self-selected)"));
        assert!(!rendered.contains("`src/unit.test.ts` ➔ [self]"));
    }

    #[test]
    fn explain_renders_empty_fallback_and_resource_without_call_sites() {
        let selected_plan = TestPlan {
            changed_files: vec!["resource.txt".to_string(), "unmatched.ts".to_string()],
            selected_tests: vec![selected(
                "resource.test.ts",
                Confidence::Low,
                "resource.txt",
                "resource",
                Some(ImpactEdgeDetail::Resource {
                    consumer_file: "resource.test.ts".to_string(),
                    call_sites: Vec::new(),
                }),
            )],
            groups: Vec::new(),
            warnings: vec![Warning {
                r#type: "fallback-warning".to_string(),
                message: "needs attention".to_string(),
                file: "config.ts".to_string(),
                line: None,
            }],
            fallback_triggered: true,
            fallback_reason: Some("configuration changed".to_string()),
            ..Default::default()
        };
        let rendered =
            super::super::render(&selected_plan, PlanFormat::Explain, "tests plan").unwrap();
        assert!(rendered.contains("- resource.txt\n- unmatched.ts"));
        assert!(rendered
            .contains("Path: `resource.txt` ➔ [resource (resource.test.ts)] ➔ `resource.test.ts`"));

        let empty_plan = TestPlan {
            changed_files: Vec::new(),
            selected_tests: Vec::new(),
            groups: Vec::new(),
            warnings: Vec::new(),
            fallback_triggered: false,
            fallback_reason: None,
            ..Default::default()
        };
        assert_eq!(
            super::super::render(&empty_plan, PlanFormat::Explain, "tests plan").unwrap(),
            "Test plan: 0 selected test(s)\nFallback: not triggered\n\nChanged files (0):\n- None\n\nNo tests selected.\n"
        );
    }

    fn selected(
        test_file: &str,
        confidence: Confidence,
        changed_file: &str,
        via: &str,
        detail: Option<ImpactEdgeDetail>,
    ) -> SelectedTest {
        SelectedTest {
            test_file: test_file.to_string(),
            confidence,
            targets: Vec::new(),
            reasons: vec![ImpactReason {
                changed_file: changed_file.to_string(),
                path: vec![changed_file.to_string(), test_file.to_string()],
                via: vec![via.to_string()],
                via_details: vec![detail],
            }],
        }
    }
}
