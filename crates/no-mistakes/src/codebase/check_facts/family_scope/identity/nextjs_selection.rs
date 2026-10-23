use super::{fixture, FamilyPathScope};
use crate::codebase::check_facts::{
    collect_check_facts, configure_nextjs_caching_scope, CheckFactPlan,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;
use crate::config::v2::Effort;

#[test]
fn nextjs_selection_materializes_membership_only_for_partial_scopes() {
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let mut files = discover_files(&root, &[]);
    files.sort();
    assert!(!files.is_empty());
    let excluded = files
        .iter()
        .position(|path| path == &root.join("web/other/excluded.ts"))
        .unwrap();
    assert!(files[..excluded].contains(&root.join("backend/excluded.ts")));
    assert!(files[excluded + 1..].contains(&root.join("web/src/selected.ts")));
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &files);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for effort in [Effort::Max, Effort::Low] {
        for (name, normalizations, findings) in [
            ("filtered-all.yml", 0, 6),
            ("low.yml", 5, 2),
            ("filtered-prefix.yml", files.len() - 1, 5),
        ] {
            let mut config =
                crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
            config.effort_override = Some(effort);
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
            let full = collect_check_facts(&root, files.clone(), plan.clone());
            super::super::nextjs::take_membership_normalizations();
            configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).unwrap();
            assert_eq!(
                super::super::nextjs::take_membership_normalizations(),
                normalizations,
                "{name}: {effort:?}"
            );
            assert_eq!(
                matches!(&plan.nextjs_caching_paths, FamilyPathScope::All),
                name == "filtered-all.yml"
            );
            let scoped = collect_check_facts(&root, files.clone(), plan);
            let report = |facts: &crate::codebase::check_facts::CheckFactMap| {
                crate::codebase::rules::nextjs_no_caching::diagnostics_with_facts_for_aggregate(
                    &root, &config, facts, None, false,
                )
                .unwrap()
            };
            let actual = report(&scoped);
            let expected = report(&full);
            assert_eq!(actual.findings.len(), findings, "{name}: {effort:?}");
            assert_eq!(
                actual.advisories.len(),
                usize::from(effort == Effort::Low),
                "{name}"
            );
            assert_eq!(actual.findings, expected.findings);
            assert_eq!(actual.advisories, expected.advisories);
        }
    }
}

#[test]
fn nextjs_filtered_empty_inventory_remains_empty_for_both_efforts() {
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("filtered-all.yml"))).unwrap();
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &[]);
    for low in [false, true] {
        let mut plan = CheckFactPlan {
            nextjs_caching: !low,
            nextjs_caching_low: low,
            ..Default::default()
        };
        super::super::nextjs::take_membership_normalizations();
        configure_nextjs_caching_scope(&root, &[], &config, &inferred, &mut plan).unwrap();
        assert!(
            matches!(&plan.nextjs_caching_paths, FamilyPathScope::Only(paths) if paths.is_empty())
        );
        assert_eq!(super::super::nextjs::take_membership_normalizations(), 0);
    }
}
