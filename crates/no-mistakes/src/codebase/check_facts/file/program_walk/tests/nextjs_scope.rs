use super::{ast, collect_fused_check_program, CheckFactPlan, Path};

#[test]
fn nextjs_scope_skips_actual_visitor_work_and_preserves_other_fused_consumers() {
    use crate::codebase::check_facts::FamilyPathScope;
    use std::sync::Arc;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-facts/family-scope-nextjs");
    let selected = root.join("web/src/selected.ts");
    let excluded = root.join("backend/excluded.ts");
    let source = std::fs::read_to_string(&excluded).unwrap();
    for low in [false, true] {
        let mut plan = CheckFactPlan {
            nextjs_caching: !low,
            nextjs_caching_low: low,
            nextjs_caching_paths: FamilyPathScope::Only(Arc::new(
                [selected.clone()].into_iter().collect(),
            )),
            ..Default::default()
        };
        let collect = |path: &Path, plan: &CheckFactPlan| {
            take_work_counts();
            let facts = ast::with_program(path, &source, |program, source| {
                collect_fused_check_program(path, source, program, plan)
            })
            .unwrap();
            (facts, take_work_counts())
        };
        let (facts, counts) = collect(&excluded, &plan);
        assert_eq!(
            counts,
            (0, 0),
            "excluded cache-only files must not prepare or walk"
        );
        assert!(facts.nextjs_caching.is_none());
        assert!(facts.nextjs_caching_low.is_none());
        let (facts, counts) = collect(&selected, &plan);
        assert_eq!(counts, (1, 1));
        let findings = if low {
            facts.nextjs_caching_low
        } else {
            facts.nextjs_caching
        };
        assert_eq!(findings.unwrap().len(), 1);
        plan.dynamic_imports = true;
        let (facts, counts) = collect(&excluded, &plan);
        assert_eq!(
            counts,
            (0, 1),
            "other consumers retain the shared walk without cache work"
        );
        assert_eq!(facts.dynamic_imports.unwrap().dynamic_imports.len(), 1);
        assert!(facts.nextjs_caching.is_none());
        assert!(facts.nextjs_caching_low.is_none());
    }
}

#[test]
fn nextjs_mixed_scope_prepares_both_collectors_in_one_walk() {
    use crate::codebase::check_facts::FamilyPathScope;
    use std::sync::Arc;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-facts/family-scope-nextjs");
    let path = root.join("web/src/selected.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let plan = CheckFactPlan {
        nextjs_caching: true,
        nextjs_caching_low: true,
        nextjs_caching_paths: FamilyPathScope::Only(Arc::new([path.clone()].into_iter().collect())),
        ..Default::default()
    };
    take_work_counts();
    let facts = ast::with_program(&path, &source, |program, source| {
        collect_fused_check_program(&path, source, program, &plan)
    })
    .unwrap();
    assert_eq!(take_work_counts(), (2, 1));
    assert_eq!(facts.nextjs_caching.unwrap().len(), 1);
    assert_eq!(facts.nextjs_caching_low.unwrap().len(), 1);
}

use std::cell::Cell;
thread_local! {
    static COUNTS: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}
pub(crate) fn record_nextjs() {
    COUNTS.with(|counts| {
        let (nextjs, walks) = counts.get();
        counts.set((nextjs + 1, walks));
    });
}
pub(crate) fn record_walk() {
    COUNTS.with(|counts| {
        let (nextjs, walks) = counts.get();
        counts.set((nextjs, walks + 1));
    });
}
pub(super) fn take_work_counts() -> (usize, usize) {
    COUNTS.with(|counts| counts.replace((0, 0)))
}
