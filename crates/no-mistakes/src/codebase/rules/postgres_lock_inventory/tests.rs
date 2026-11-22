use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::config::v2::schema::RuleScope;

fn fixture(scenario: &str) -> PathBuf {
    crate::test_support::fixture_root("postgres-lock-inventory-core").join(scenario)
}

fn report(scenario: &str, paths: &[&str]) -> LockInventoryReport {
    report_with_prepared_facts(scenario, paths, |_, _, _| {})
}

fn report_with_prepared_facts(
    scenario: &str,
    paths: &[&str],
    inspect: impl FnOnce(&Path, &CheckFactMap, &NoMistakesConfig),
) -> LockInventoryReport {
    let root = fixture(scenario);
    let files: Vec<_> = paths.iter().map(|file| root.join(file)).collect();
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.into(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(
            "importSpecifier: '@example/db'\nhelpers: [reserveJob]",
        ),
        ..Default::default()
    });
    let sources = super::super::source_store_for_files(&files);
    let facts = super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID)
        .expect("prepare inventory fixture");
    inspect(&root, &facts, &config);
    inventory_with_files_sources_and_facts(&root, &config, &files, &sources, &facts)
        .expect("collect inventory fixture")
}

fn baseline(report: &LockInventoryReport) -> LockInventoryBaseline {
    LockInventoryBaseline {
        version: 1,
        sites: report
            .sites
            .iter()
            .map(|site| LockInventoryBaselineSite {
                file: site.file.clone(),
                category: site.category.clone(),
                key: site.key.clone(),
            })
            .collect(),
    }
}

#[test]
fn fingerprints_survive_movement_comments_and_whitespace() {
    let paths = ["locks.sql", "queries.ts"];
    let before = report("before", &paths);
    let mut moved = report("moved", &paths);
    assert!(before.unanalyzed.is_empty());
    assert!(moved.unanalyzed.is_empty());
    assert_eq!(before.counts["for-update"], 2);
    assert_eq!(before.counts["for-key-share"], 1);
    assert_eq!(before.counts["for-share"], 1);
    assert_eq!(before.counts["helper"], 3);
    assert_eq!(before.counts, moved.counts);
    let keys: BTreeSet<_> = before.sites.iter().map(|site| &site.key).collect();
    assert_eq!(keys.len(), before.sites.len());
    moved.added.clear();
    baseline::compare(&mut moved, Some(&baseline(&before)));
    assert!(moved.added.is_empty());
    assert!(moved.removed.is_empty());
}

#[test]
fn substitutes_by_identity_and_defers_removals_when_incomplete() {
    let before = report("before", &["locks.sql"]);
    let baseline = baseline(&before);
    let mut replaced = report("replaced", &["locks.sql"]);
    replaced.added.clear();
    baseline::compare(&mut replaced, Some(&baseline));
    assert_eq!(before.counts, replaced.counts);
    assert_eq!(replaced.added.len(), 1);
    assert_eq!(replaced.removed.len(), 1);
    assert_eq!(report_findings(replaced).len(), 2);

    let mut incomplete = report("incomplete", &["queries.ts"]);
    incomplete.added.clear();
    baseline::compare(&mut incomplete, Some(&baseline));
    assert_eq!(incomplete.unanalyzed.len(), 1);
    assert_eq!(incomplete.added.len(), 1);
    assert!(incomplete.removed.is_empty());
}

#[test]
fn stored_routine_body_excludes_uncertain_sql_and_defers_removals() {
    let before = report("before", &["locks.sql"]);
    let mut routine = report("stored-routine", &["locks.sql"]);
    assert!(routine.sites.is_empty());
    assert!(routine.counts.is_empty());
    assert_eq!(
        routine.unanalyzed,
        vec![UnanalyzedLockSite {
            file: "locks.sql".into(),
            line: 2,
            reason: "stored routine body could not be resolved".into(),
        }]
    );

    baseline::compare(&mut routine, Some(&baseline(&before)));
    assert!(routine.removed.is_empty());
}

#[test]
fn reused_imported_sql_uses_each_executor_location_and_callbacks_count_only_calls() {
    use crate::codebase::postgres::statements::SqlFactSite;

    let report = report_with_prepared_facts(
        "imported",
        &["queries.mts", "lock-clause.mts"],
        |root, facts, config| {
            let options = CompiledOptions::from_rule(&config.rules[0]).unwrap();
            let statements = facts
                .postgres_statements(&root.join("queries.mts"), Some(&options.embedded))
                .unwrap();
            let origin = statements
                .iter()
                .filter_map(|statement| statement.variant_locations.as_ref())
                .find(|locations| locations.original_call_line == 29)
                .and_then(|locations| locations.position(SqlFactSite::Function(0)))
                .expect("prepared imported SQL helper position");
            assert_eq!(
                origin.source_path.as_deref(),
                Some(root.join("lock-clause.mts").as_path())
            );
            assert_eq!(origin.source_line, 5);
        },
    );
    assert!(report.unanalyzed.is_empty(), "{:?}", report.unanalyzed);
    assert_eq!(report.counts["for-update"], 3);
    assert_eq!(report.counts["helper"], 4);
    let helper_sites: Vec<_> = report
        .sites
        .iter()
        .filter(|site| site.category == "helper")
        .map(|site| (site.file.as_str(), site.line))
        .collect();
    assert_eq!(
        helper_sites,
        [
            ("queries.mts", 25),
            ("queries.mts", 29),
            ("queries.mts", 32),
            ("queries.mts", 35)
        ]
    );
    let executor_sites: Vec<_> = report
        .sites
        .iter()
        .filter(|site| site.category == "for-update")
        .map(|site| (site.file.as_str(), site.line))
        .collect();
    assert_eq!(
        executor_sites,
        [
            ("queries.mts", 10),
            ("queries.mts", 16),
            ("queries.mts", 22)
        ]
    );
}

#[test]
fn distinct_imported_helpers_in_complete_alternatives_keep_both_sites() {
    let report = report("imported", &["alternatives.mts", "lock-clause.mts"]);
    assert!(report.unanalyzed.is_empty(), "{:?}", report.unanalyzed);
    let sites: Vec<_> = report
        .sites
        .iter()
        .map(|site| (site.file.as_str(), site.line, site.category.as_str()))
        .collect();
    assert_eq!(sites, [("alternatives.mts", 5, "helper"); 2]);
    assert_ne!(report.sites[0].key, report.sites[1].key);
}

#[test]
fn scopes_helper_only_and_native_applications_to_their_own_baselines() {
    let root = fixture("scoped");
    let standalone = crate::postgres_lock_inventory::run(
        &crate::postgres_lock_inventory::PostgresLockInventoryOptions {
            root: root.clone(),
            config: Some("config.yml".into()),
            write_baseline: false,
        },
    );
    assert!(standalone.unwrap_err().to_string().contains("exactly one"));
    let files = ["app.ts", "excluded.ts", "locks.sql", "excluded.sql"].map(|file| root.join(file));
    let session = AnalysisSession::new(None);
    let config = session
        .config(&root, Some(Path::new("config.yml")))
        .unwrap();
    assert_eq!(
        crate::codebase::postgres::helper_call_site_files(&root, &config, &files).unwrap(),
        vec![root.join("app.ts")]
    );
    let sources = super::super::source_store_for_files(&files);
    let facts =
        super::super::prepare_sql_rule_facts(&root, &config, &files, &sources, RULE_ID).unwrap();
    let check =
        |config| check_with_files_sources_and_facts(&root, config, &files, &sources, &facts);
    let findings = check(&config).unwrap();
    let sites: Vec<_> = findings
        .iter()
        .map(|finding| (finding.file.as_str(), finding.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        sites,
        [
            ("app.ts", "helper"),
            ("helpers.json", "stale-baseline"),
            ("locks.sql", "for-update")
        ]
    );

    let mut duplicate = config.as_ref().clone();
    duplicate.rules[1].options["baselineFile"] = serde_yaml::Value::String("./helpers.json".into());
    assert!(check(&duplicate)
        .unwrap_err()
        .to_string()
        .contains("distinct"));
}

#[test]
fn inventories_complete_alternatives_and_reports_sampled_and_opaque_sql() {
    let report = report("variants", &["queries.ts", "broken.ts"]);
    assert_eq!(
        report.counts,
        std::collections::BTreeMap::from([("for-share".into(), 1)])
    );
    assert_eq!(report.sites[0].line, 6);
    let gaps: Vec<_> = report
        .unanalyzed
        .iter()
        .map(|gap| (gap.file.as_str(), gap.line, gap.reason.as_str()))
        .collect();
    assert_eq!(
        gaps,
        [
            ("broken.ts", 1, "TypeScript source could not be analyzed"),
            ("queries.ts", 11, "executor SQL alternatives are incomplete"),
            ("queries.ts", 15, "executor SQL could not be resolved"),
        ]
    );
}
