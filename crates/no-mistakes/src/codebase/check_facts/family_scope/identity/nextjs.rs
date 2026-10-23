use super::{applies, fixture, FamilyPathScope};
use crate::codebase::check_facts::configure_nextjs_caching_scope;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;

#[test]
fn nextjs_scope_preserves_findings_and_exact_application_filters() {
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let files = discover_files(&root, &[]);
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &files);
    let yml = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&yml)).unwrap();
    let full = CheckFactPlan {
        nextjs_caching: true,
        source: true,
        ..Default::default()
    };
    let mut scoped = full.clone();
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut scoped).unwrap();
    assert!(applies(
        &scoped.nextjs_caching_paths,
        &root.join("web/src/selected.ts")
    ));
    for rel in [
        "backend/excluded.ts",
        "web/other/excluded.ts",
        "web/src/project-excluded.ts",
        "web/src/generated/excluded.ts",
    ] {
        assert!(
            !applies(&scoped.nextjs_caching_paths, &root.join(rel)),
            "{rel}"
        );
    }
    for rel in ["web/src/disabled.ts", "web/src/skipped/selected.ts"] {
        assert!(
            applies(&scoped.nextjs_caching_paths, &root.join(rel)),
            "{rel}"
        );
    }
    let mut reverse = full.clone();
    reverse.include(scoped.clone());
    assert_eq!(reverse.nextjs_caching_paths, FamilyPathScope::All);
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped.clone());
    let full_facts = collect_check_facts(&root, files, full.clone());
    let report = |facts: &crate::codebase::check_facts::CheckFactMap| {
        let diagnostics =
            crate::codebase::rules::nextjs_no_caching::diagnostics_with_facts_for_aggregate(
                &root, &config, facts, None, false,
            )
            .unwrap();
        (diagnostics.findings, diagnostics.advisories)
    };
    let (findings, advisories) = report(&scoped_facts);
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.file == "web/src/selected.ts"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "web/src/skipped/selected.ts"));
    assert!(advisories.is_empty());
    assert_eq!((findings, advisories), report(&full_facts));
    // Unrestricted direct callers must widen a scoped merged plan in either order.
    scoped.include(full.clone());
    assert_eq!(scoped.nextjs_caching_paths, FamilyPathScope::All);
}

#[test]
fn nextjs_repository_demand_keeps_all_without_hiding_invalid_filters() {
    use crate::config::v2::schema::{RuleDef, RuleScope};
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let files = discover_files(&root, &[]);
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &files);
    let mut config = crate::config::v2::NoMistakesConfig::default();
    let mut disabled = CheckFactPlan::default();
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut disabled).unwrap();
    assert_eq!(disabled.nextjs_caching_paths, FamilyPathScope::All);
    let mut unconfigured = CheckFactPlan {
        nextjs_caching: true,
        ..Default::default()
    };
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut unconfigured).unwrap();
    assert_eq!(unconfigured.nextjs_caching_paths, FamilyPathScope::All);
    config.rules.push(RuleDef {
        rule: "nextjs-no-caching".into(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    let mut plan = CheckFactPlan {
        nextjs_caching: true,
        ..Default::default()
    };
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).unwrap();
    assert_eq!(plan.nextjs_caching_paths, FamilyPathScope::All);
    // A broad application must not swallow a later application's invalid glob.
    config.rules.push(RuleDef {
        rule: "nextjs-no-caching".into(),
        scope: Some(RuleScope::Repository),
        include: vec!["[".into()],
        ..Default::default()
    });
    assert!(configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).is_err());
}

#[test]
fn nextjs_applications_union_and_empty_selection_stays_empty() {
    use crate::config::v2::schema::{RuleDef, RuleScope};
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let files = discover_files(&root, &[]);
    let inferred = crate::codebase::config::InferredRoots::from_visible(&root, &files);
    let mut config = crate::config::v2::NoMistakesConfig::default();
    let rule = |include: &str| RuleDef {
        rule: "nextjs-no-caching".into(),
        scope: Some(RuleScope::Repository),
        include: vec![include.into()],
        ..Default::default()
    };
    config.rules.push(rule("missing/**"));
    let mut plan = CheckFactPlan {
        nextjs_caching: true,
        ..Default::default()
    };
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).unwrap();
    assert!(matches!(&plan.nextjs_caching_paths, FamilyPathScope::Only(paths) if paths.is_empty()));
    config.rules.push(rule("web/src/selected.ts"));
    config.rules.push(rule("backend/excluded.ts"));
    configure_nextjs_caching_scope(&root, &files, &config, &inferred, &mut plan).unwrap();
    assert!(applies(
        &plan.nextjs_caching_paths,
        &root.join("web/src/selected.ts")
    ));
    assert!(applies(
        &plan.nextjs_caching_paths,
        &root.join("backend/excluded.ts")
    ));
    assert!(!applies(
        &plan.nextjs_caching_paths,
        &root.join("web/src/generated/excluded.ts")
    ));
}

#[test]
fn nextjs_standalone_preserves_skipped_primary_file_behavior() {
    let root = fixture("check-facts/family-scope-nextjs");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("graph.yml")), None).unwrap();
    let cache: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == "nextjs-no-caching")
        .collect();
    assert_eq!(cache.len(), 1, "{cache:?}");
    assert!(!cache
        .iter()
        .any(|finding| finding.file == "web/src/skipped/selected.ts"));
}

#[test]
fn nextjs_scope_uses_supplied_framework_inference_without_rediscovery() {
    use crate::codebase::config::InferredRoots;
    use crate::config::v2::schema::ProjectType;
    let root = normalize_path(&fixture("check-facts/family-scope-nextjs"));
    let files = discover_files(&root, &[]);
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let project = config.projects.get_mut("web").unwrap();
    project.root = None;
    project.type_ = Some(ProjectType::Nextjs);
    let mut plan = CheckFactPlan {
        nextjs_caching: true,
        ..Default::default()
    };
    // The physical next.config.ts exists; a prepared negative must not rediscover it.
    configure_nextjs_caching_scope(
        &root,
        &files,
        &config,
        &InferredRoots::from_visible(&root, &[]),
        &mut plan,
    )
    .unwrap();
    assert!(matches!(&plan.nextjs_caching_paths, FamilyPathScope::Only(paths) if paths.is_empty()));
    configure_nextjs_caching_scope(
        &root,
        &files,
        &config,
        &InferredRoots::from_visible(&root, &files),
        &mut plan,
    )
    .unwrap();
    assert!(applies(
        &plan.nextjs_caching_paths,
        &root.join("web/src/selected.ts")
    ));
}
