use super::collect_fused_check_program;
use crate::ast;
use crate::codebase::check_facts::CheckFactPlan;
use std::path::Path;

fn fused(source: &str, plan: &CheckFactPlan) -> super::FusedCheckFacts {
    let path = Path::new("src/page.ts");
    ast::with_program(path, source, |program, src| {
        collect_fused_check_program(path, src, program, plan)
    })
    .expect("source should parse")
}

#[test]
fn fused_walk_skips_when_no_check_only_flags() {
    let facts = fused("export const value = 1;", &CheckFactPlan::default());
    assert!(facts.dynamic_imports.is_none());
    assert!(facts.nextjs_caching.is_none());
    assert!(facts.storybook.is_none());
}

#[test]
fn fused_walk_records_storybook_identifiers() {
    let plan = CheckFactPlan {
        storybook: true,
        ..CheckFactPlan::default()
    };
    let facts = fused(
        "import { Button } from './Button';\nexport const story = Button;\n",
        &plan,
    );
    let storybook = facts.storybook.expect("storybook facts");
    assert!(!storybook.used_runtime_imports.is_empty());
    assert!(facts.dynamic_imports.is_none());
    assert!(facts.nextjs_caching.is_none());
}

#[test]
fn fused_walk_records_dynamic_import_expressions_and_calls() {
    let plan = CheckFactPlan {
        dynamic_imports: true,
        ..CheckFactPlan::default()
    };
    let facts = fused(
        "export async function load() { await import('./mod'); require('./cjs'); }\n",
        &plan,
    );
    let dynamic = facts.dynamic_imports.expect("dynamic import facts");
    assert!(!dynamic.dynamic_imports.is_empty());
}

#[test]
fn fused_walk_visits_nextjs_caching_nodes() {
    let plan = CheckFactPlan {
        nextjs_caching: true,
        ..CheckFactPlan::default()
    };
    let source = r#"
        import { unstable_cache } from "next/cache";
        export { unstable_cache as cache };
        export function helper() {}
        export default function Page() {
          "use cache";
          fetch("/x");
          cache = unstable_cache;
          return null;
        }
    "#;
    let facts = fused(source, &plan);
    assert!(facts.nextjs_caching.is_some());
    assert!(facts.dynamic_imports.is_none());
    assert!(facts.storybook.is_none());
}

#[test]
fn fused_walk_can_enable_all_three_check_only_flags() {
    let plan = CheckFactPlan {
        dynamic_imports: true,
        nextjs_caching: true,
        storybook: true,
        ..CheckFactPlan::default()
    };
    let source = r#"
        import { cache } from "react";
        export default function Page() {
          import("./mod");
          return Label;
        }
    "#;
    let facts = fused(source, &plan);
    assert!(facts.dynamic_imports.is_some());
    assert!(facts.nextjs_caching.is_some());
    assert!(facts.storybook.is_some());
}

#[test]
fn fused_walk_matches_saved_nextjs_shadowing_findings() {
    let mut failures = Vec::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching/shadowing");
    for case in [
        "catch",
        "declarations",
        "var",
        "loops",
        "switch",
        "parameters",
    ] {
        let path = root.join(format!("{case}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let plan = CheckFactPlan {
            nextjs_caching: true,
            ..CheckFactPlan::default()
        };
        let findings = fused(&source, &plan).nextjs_caching.unwrap();
        let standalone =
            crate::codebase::rules::nextjs_no_caching::extract(&path, &source).unwrap();
        assert_eq!(findings, standalone, "{case}: standalone/fused parity");
        let expected: Vec<_> = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| line.contains("// finding").then_some(index + 1))
            .collect();
        let actual: Vec<_> = findings.iter().map(|finding| finding.line).collect();
        if actual != expected {
            failures.push(format!("{case}: actual={actual:?}, expected={expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn fused_nextjs_scopes_preserve_other_fact_families() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching/shadowing/mixed.ts");
    let source = std::fs::read_to_string(path).unwrap();
    let expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.split_once("// dynamic ")
                .map(|(_, name)| (Some(name.to_string()), line + 1))
        })
        .collect();
    let mut references = Vec::new();
    for nextjs_caching in [false, true] {
        let plan = CheckFactPlan {
            nextjs_caching,
            dynamic_imports: true,
            storybook: true,
            ..Default::default()
        };
        let facts = fused(&source, &plan);
        let dynamic: Vec<_> = facts
            .dynamic_imports
            .unwrap()
            .dynamic_imports
            .into_iter()
            .map(|import| (import.specifier, import.line))
            .collect();
        assert_eq!(dynamic, expected);
        let storybook = facts.storybook.unwrap();
        assert_eq!(storybook.side_effect_imports.len(), 1);
        assert_eq!(storybook.side_effect_imports[0].source, "./setup");
        assert_eq!(
            storybook
                .used_runtime_imports
                .iter()
                .map(|import| import.local.as_str())
                .collect::<Vec<_>>(),
            [
                "Of",
                "In",
                "For",
                "Discriminant",
                "Case",
                "Body",
                "Caught",
                "Base",
                "Static"
            ]
        );
        references.push(storybook.used_runtime_imports);
        assert_eq!(
            facts.nextjs_caching.map(|findings| findings.len()),
            nextjs_caching.then_some(1)
        );
    }
    assert_eq!(references[0], references[1]);
}

mod nextjs_scope;
pub(super) use nextjs_scope::{record_nextjs, record_walk};
