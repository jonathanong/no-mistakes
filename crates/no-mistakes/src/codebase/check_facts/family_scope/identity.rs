use super::{applies, configure_family_path_scopes, FamilyPathScope};
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;
use crate::codebase::unique_exports::UniqueExportFinding;
use std::path::{Path, PathBuf};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(rel)
}

fn queue_plan(root: &Path) -> CheckFactPlan {
    let config = crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    CheckFactPlan {
        queue: true,
        queue_factory_names: config.queues.factories.queue_names(),
        queue_factory_roles: config
            .queues
            .factories
            .clone()
            .with_project_wait_scopes(&config),
        ..Default::default()
    }
}

#[test]
fn queue_memmem_scope_preserves_factory_reexport_facts() {
    let root = fixture("queue-ast-hop/factory-reexports/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    assert_eq!(
        scoped.queue_paths,
        FamilyPathScope::All,
        "enq/unresolved.mts imports @external/queues, which the relative closure cannot follow"
    );
    assert_eq!(
        queue_report(&root, collect_check_facts(&root, files.clone(), scoped)),
        queue_report(&root, collect_check_facts(&root, files, queue_plan(&root))),
    );
}

fn queue_report(root: &Path, facts: crate::codebase::check_facts::CheckFactMap) -> String {
    let report = crate::queue::analyze_project_with_facts(root, None, &[], &facts).unwrap();
    format!(
        "{:#?}\n{:#?}\n{:#?}",
        report.check, report.edges, report.diagnostics
    )
}

#[test]
fn unique_exports_include_scope_preserves_fixture_findings() {
    let root = fixture("config-v2/rule-path-filters/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports fixture config");
    let mut scoped = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let unrestricted = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped);
    let full_facts = collect_check_facts(&root, files, unrestricted);
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &scoped_facts,
    )
    .unwrap();
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &full_facts,
    )
    .unwrap();
    assert_eq!(scoped_findings, full_findings);
}

#[test]
fn unique_exports_project_scope_keeps_reexport_origin_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-unique-canonical");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports suppression fixture config");
    let mut scoped = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let origin =
        crate::codebase::ts_resolver::normalize_path(&root.join("shared/collision-origin.ts"));
    assert!(
        crate::codebase::check_facts::family_scope::applies(&scoped.symbols_paths, &origin),
        "reexport origins outside projects must stay in the symbols closure"
    );
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &collect_check_facts(&root, files.clone(), scoped),
    )
    .expect("scoped unique-exports should keep reexport origin facts");
    let full_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        None,
        &collect_check_facts(
            &root,
            files,
            CheckFactPlan {
                symbols: true,
                source: true,
                ..Default::default()
            },
        ),
    )
    .unwrap();
    assert_eq!(scoped_findings, full_findings);
}

#[test]
fn import_closure_falls_back_to_all_on_path_aliases() {
    let root = fixture("check-facts/family-scope-alias");
    assert_eq!(closure_from(&root, "src/entry.ts"), FamilyPathScope::All);
}

#[test]
fn import_closure_falls_back_to_all_on_scoped_package_specifiers() {
    let root = fixture("check-facts/family-scope-alias");
    assert_eq!(closure_from(&root, "src/scoped.ts"), FamilyPathScope::All);
}

fn closure_from(root: &Path, rel: &str) -> FamilyPathScope {
    let files = discover_files(root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let entry = normalize_path(&root.join(rel));
    super::closure::with_import_closure(
        root,
        &files,
        sources.as_ref(),
        [entry].into_iter().collect(),
    )
}

#[test]
fn import_closure_keeps_relative_cycles_restricted() {
    let root = fixture("check-facts/family-scope-cycle");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let start = crate::codebase::ts_resolver::normalize_path(&root.join("src/a.ts"));
    let scope = super::closure::with_import_closure(
        &root,
        &files,
        sources.as_ref(),
        [start].into_iter().collect(),
    );
    assert!(matches!(scope, FamilyPathScope::Only(_)), "{scope:?}");
}

#[test]
fn empty_import_seeds_stay_all() {
    let root = fixture("check-facts/family-scope-alias");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    assert_eq!(
        super::closure::with_import_closure(&root, &files, sources.as_ref(), Default::default()),
        FamilyPathScope::All
    );
}

#[test]
fn symbols_scope_unions_storybook_seeds() {
    let root = fixture("rules/require-storybook-stories/fixture/covered");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("storybook fixture config");
    let mut plan = CheckFactPlan {
        symbols: true,
        storybook: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut plan).unwrap();
    assert_eq!(plan.storybook_paths, plan.symbols_paths);
}

#[test]
fn unique_exports_workspace_alias_preserves_fixture_findings() {
    let root = fixture("codebase-analysis/unique-exports-project-scope-workspace/fixture");
    let (scoped, full) = unique_exports_scoped_and_full(&root);
    let origin = normalize_path(&root.join("packages/source/src/index.ts"));
    assert!(
        applies(&scoped.0, &origin),
        "workspace package reexport origins must keep symbol facts"
    );
    assert_eq!(scoped.1, full);
}

#[test]
fn storybook_top_level_include_keeps_configured_story_facts() {
    let root = fixture("check-facts/family-scope-storybook");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("storybook include fixture config");
    let mut scoped = CheckFactPlan {
        react: true,
        symbols: true,
        storybook: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let story = normalize_path(&root.join("stories/button.stories.tsx"));
    assert!(
        applies(&scoped.storybook_paths, &story),
        "configured stories must stay in the Storybook extraction scope"
    );
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped);
    let full_facts = collect_check_facts(
        &root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            source: true,
            ..Default::default()
        },
    );
    let scoped_story = format!(
        "{:?}",
        scoped_facts
            .ts
            .get(&story)
            .and_then(|facts| facts.storybook.as_ref())
    );
    let full_story = format!(
        "{:?}",
        full_facts
            .ts
            .get(&story)
            .and_then(|facts| facts.storybook.as_ref())
    );
    assert_ne!(
        scoped_story, "None",
        "scoped collection must extract story facts"
    );
    assert_eq!(scoped_story, full_story);
}

#[test]
fn queue_wait_routes_stay_in_memmem_scope() {
    let root = fixture("queue-ast-hop/factory-roles/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let wait = normalize_path(&root.join("routes/wait.ts"));
    assert!(
        applies(&scoped.queue_paths, &wait),
        "addAndWait routes must stay in the queue extraction scope"
    );
    assert_eq!(
        queue_report(&root, collect_check_facts(&root, files.clone(), scoped)),
        queue_report(&root, collect_check_facts(&root, files, queue_plan(&root))),
    );
}

fn unique_exports_scoped_and_full(
    root: &Path,
) -> (
    (FamilyPathScope, Vec<UniqueExportFinding>),
    Vec<UniqueExportFinding>,
) {
    let files = discover_files(root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml")))
        .expect("unique-exports fixture config");
    let mut scoped = CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let yml = root.join(".no-mistakes.yml");
    let scoped_findings = crate::codebase::unique_exports::analyze_project_with_facts(
        root,
        Some(&yml),
        None,
        &collect_check_facts(root, files.clone(), scoped.clone()),
    )
    .unwrap();
    let full = crate::codebase::unique_exports::analyze_project_with_facts(
        root,
        Some(&yml),
        None,
        &collect_check_facts(
            root,
            files,
            CheckFactPlan {
                symbols: true,
                source: true,
                ..Default::default()
            },
        ),
    )
    .unwrap();
    ((scoped.symbols_paths, scoped_findings), full)
}
