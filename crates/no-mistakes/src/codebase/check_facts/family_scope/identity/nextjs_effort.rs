use super::{applies, fixture, FamilyPathScope};
use crate::codebase::check_facts::{
    collect_check_facts, configure_nextjs_caching_scope, CheckFactPlan,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;

#[test]
fn nextjs_effort_scopes_preserve_findings_and_suppression() {
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let files = discover_files(&root, &[]);
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &files);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for (name, count, max, low) in [
        ("graph.yml", 2, true, false),
        ("low.yml", 2, false, true),
        ("mixed.yml", 3, true, true),
        ("low-empty.yml", 0, false, true),
    ] {
        let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        // Unprepared low requests intentionally retain max compatibility.
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            &root,
            &mut config,
            &files,
            sources.as_ref(),
        )
        .unwrap();
        let mut plan = CheckFactPlan {
            source: true,
            ..Default::default()
        };
        crate::codebase::rules::nextjs_no_caching::configure_effort_plan(&config, &mut plan)
            .unwrap();
        assert_eq!(
            (plan.nextjs_caching, plan.nextjs_caching_low),
            (max, low),
            "{name}"
        );
        let full = collect_check_facts(&root, files.clone(), plan.clone());
        configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).unwrap();
        assert!(!applies(
            &plan.nextjs_caching_paths,
            &root.join("web/src/generated/excluded.ts")
        ));
        assert_eq!(
            applies(
                &plan.nextjs_caching_paths,
                &root.join("backend/excluded.ts")
            ),
            name == "mixed.yml"
        );
        assert_eq!(
            applies(
                &plan.nextjs_caching_paths,
                &root.join("web/src/selected.ts")
            ),
            name != "low-empty.yml"
        );
        let scoped = collect_check_facts(&root, files.clone(), plan);
        let report = |facts: &crate::codebase::check_facts::CheckFactMap| {
            crate::codebase::rules::nextjs_no_caching::diagnostics_with_facts_for_aggregate(
                &root, &config, facts, None, false,
            )
            .unwrap()
        };
        let diagnostics = report(&scoped);
        let full_diagnostics = report(&full);
        let findings = &diagnostics.findings;
        assert_eq!(findings.len(), count, "{name}: {findings:?}");
        assert_eq!(findings, &full_diagnostics.findings, "{name}");
        assert_eq!(
            diagnostics.advisories, full_diagnostics.advisories,
            "{name}"
        );
        let advisory_files: Vec<_> = diagnostics
            .advisories
            .iter()
            .map(|f| f.file.as_str())
            .collect();
        let expected_advisories = if low && name != "low-empty.yml" {
            vec!["web/src/advisory.ts"]
        } else {
            vec![]
        };
        assert_eq!(advisory_files, expected_advisories, "{name}");
        assert!(!findings.iter().any(|f| f.file == "web/src/disabled.ts"));
    }
}

#[test]
fn nextjs_scopes_merge_all_effort_owners_in_both_orders() {
    use std::sync::Arc;
    let path = fixture("check-facts/family-scope-nextjs/web/src/selected.ts");
    let scope = FamilyPathScope::Only(Arc::new([path].into_iter().collect()));
    for scoped_low in [false, true] {
        let scoped = CheckFactPlan {
            nextjs_caching: !scoped_low,
            nextjs_caching_low: scoped_low,
            nextjs_caching_paths: scope.clone(),
            ..Default::default()
        };
        // A disabled plan has no scope ownership, including with low-only demand.
        let mut disabled_first = CheckFactPlan::default();
        disabled_first.include(scoped.clone());
        assert_eq!(disabled_first.nextjs_caching_paths, scope);
        let mut disabled_last = scoped.clone();
        disabled_last.include(CheckFactPlan::default());
        assert_eq!(disabled_last.nextjs_caching_paths, scope);
        for direct_low in [false, true] {
            let direct = CheckFactPlan {
                nextjs_caching: !direct_low,
                nextjs_caching_low: direct_low,
                ..Default::default()
            };
            let mut left = scoped.clone();
            left.include(direct.clone());
            let mut right = direct;
            right.include(scoped.clone());
            assert_eq!(left.nextjs_caching_paths, FamilyPathScope::All);
            assert_eq!(right.nextjs_caching_paths, FamilyPathScope::All);
        }
    }
}
