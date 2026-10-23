pub(super) use super::{applies, configure_family_path_scopes, FamilyPathScope};
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;
use crate::codebase::unique_exports::UniqueExportFinding;
use std::path::{Path, PathBuf};

mod closure;
mod nextjs;
mod queue;
mod storybook;
mod unique;

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

fn queue_report(root: &Path, facts: crate::codebase::check_facts::CheckFactMap) -> String {
    let report = crate::queue::analyze_project_with_facts(root, None, &[], &facts).unwrap();
    format!(
        "{:#?}\n{:#?}\n{:#?}",
        report.check, report.edges, report.diagnostics
    )
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

fn unique_exports_plan() -> CheckFactPlan {
    CheckFactPlan {
        symbols: true,
        source: true,
        ..Default::default()
    }
}
