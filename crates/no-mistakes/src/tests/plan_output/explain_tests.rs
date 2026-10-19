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
    let rendered = super::super::render(&selected_plan, PlanFormat::Explain, "tests plan").unwrap();
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

#[test]
fn explain_and_markdown_render_missing_dependency_baselines() {
    let plan = TestPlan {
        changed_files: vec!["package.json".to_string(), "pnpm-lock.yaml".to_string()],
        selected_tests: Vec::new(),
        groups: Vec::new(),
        warnings: vec![Warning {
            r#type: "lockfile-no-baseline".to_string(),
            message: "no baseline".to_string(),
            file: "pnpm-lock.yaml".to_string(),
            line: None,
        }],
        fallback_triggered: true,
        fallback_reason: Some(
            "Vitest setup dependencies could not be resolved statically; selected owning project tests"
                .to_string(),
        ),
        incomplete: vec![crate::tests::PlanIncompleteness {
            kind: crate::tests::PlanIncompletenessKind::DependencyBaselineMissing,
            files: vec!["package.json".to_string(), "pnpm-lock.yaml".to_string()],
        }],
        ..Default::default()
    };

    let explained = super::super::render(&plan, PlanFormat::Explain, "tests plan").unwrap();
    assert!(explained.contains("Fallback: triggered"));
    assert!(explained.contains(
        "Incomplete dependency coverage:\n- dependency-baseline-missing (package.json, pnpm-lock.yaml)\nPass --base <pre-change revision>"
    ));
    assert!(explained.contains("- lockfile-no-baseline (pnpm-lock.yaml): no baseline"));

    let markdown = super::super::render(&plan, PlanFormat::Markdown, "tests plan").unwrap();
    assert!(markdown.contains("**Incomplete dependency coverage**"));
    assert!(markdown.contains("**dependency-baseline-missing**: `package.json`, `pnpm-lock.yaml`"));
    assert!(markdown.contains("`--base <pre-change revision>`"));
    assert!(markdown.contains("**Fallback Triggered**"));
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
