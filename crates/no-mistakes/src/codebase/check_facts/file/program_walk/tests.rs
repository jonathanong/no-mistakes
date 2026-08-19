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
