use crate::codebase::dependencies::graph::{
    collect_language_frontend_edges_for_bench, count_queue_glob_matches, EdgeKind,
    LanguageFrontendEdgeRequest,
};
use crate::codebase::lang_frontends::{collect_all_lang_facts, LangFactMap, LangFrontendConfig};
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use std::path::PathBuf;
use std::sync::Arc;

/// Composed `fixtures/lang-frontends` trees plus the production collect config.
#[derive(Clone)]
pub struct LanguageFrontendFixture {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    languages: LangFrontendConfig,
    queue_enqueues: Vec<String>,
    queue_workers: Vec<String>,
    queue_cluster: Option<String>,
}

/// Stable counts from the production language-frontend collectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LanguageFrontendSummary {
    pub files: usize,
    pub parsed_files: usize,
    pub imports: usize,
    pub enqueues: usize,
    pub workers: usize,
    pub route_handlers: usize,
    pub edges: usize,
    pub queue_edges: usize,
    pub glob_matches: usize,
}

pub fn language_frontend_fixture() -> LanguageFrontendFixture {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends"),
    );
    let repo = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    let mut files = discover_visible_paths(&repo)
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&repo.join(path)))
        .filter(|path| path.starts_with(&root))
        .collect::<Vec<_>>();
    files.sort();
    LanguageFrontendFixture {
        root,
        files,
        languages: LangFrontendConfig {
            python_packages: vec![
                "python-celery-django/app".into(),
                "python-flask-fastapi".into(),
            ],
            go_modules: vec![
                "go-asynq".into(),
                "go-asynq/worker".into(),
                "go-http".into(),
            ],
            rust_packages: vec![
                "rust-mods".into(),
                "rust-mods/src".into(),
                "rust-http".into(),
            ],
            rails_apps: vec!["rails-jobs".into()],
            php_apps: vec!["php-laravel".into()],
            php_framework: Some("laravel".into()),
            java_packages: vec!["java-spring".into()],
            kotlin_packages: vec!["kotlin-spring".into()],
            elixir_apps: vec!["phoenix-routes".into()],
        },
        queue_enqueues: vec!["**/*".into()],
        queue_workers: vec!["**/*".into()],
        queue_cluster: Some("orders".into()),
    }
}

pub fn collect_language_frontend_facts(
    fixture: &LanguageFrontendFixture,
) -> LanguageFrontendSummary {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&fixture.files)));
    let facts = collect_all_lang_facts(&fixture.root, &fixture.files, &fixture.languages, &sources);
    let maps = [
        &facts.python,
        &facts.go,
        &facts.rust,
        &facts.ruby,
        &facts.php,
        &facts.java,
        &facts.kotlin,
        &facts.elixir,
    ];
    LanguageFrontendSummary {
        files: fixture.files.len(),
        parsed_files: maps.iter().map(|map| map.files.len()).sum(),
        imports: fact_len(maps, |file| file.imports.len()),
        enqueues: fact_len(maps, |file| file.queue_enqueues.len()),
        workers: fact_len(maps, |file| file.queue_workers.len()),
        route_handlers: fact_len(maps, |file| file.route_handlers.len()),
        ..LanguageFrontendSummary::default()
    }
}

pub fn collect_language_frontend_edges(
    fixture: &LanguageFrontendFixture,
) -> LanguageFrontendSummary {
    let edges = collect_language_frontend_edges_for_bench(LanguageFrontendEdgeRequest {
        root: &fixture.root,
        all_files: &fixture.files,
        languages: &fixture.languages,
        queue_enqueues: &fixture.queue_enqueues,
        queue_workers: &fixture.queue_workers,
        queue_cluster: fixture.queue_cluster.clone(),
    });
    LanguageFrontendSummary {
        files: fixture.files.len(),
        edges: edges.len(),
        queue_edges: edges
            .iter()
            .filter(|(_, _, kind)| matches!(*kind, EdgeKind::QueueEnqueue | EdgeKind::QueueWorker))
            .count(),
        ..LanguageFrontendSummary::default()
    }
}

pub fn match_language_frontend_queue_globs(
    fixture: &LanguageFrontendFixture,
) -> LanguageFrontendSummary {
    LanguageFrontendSummary {
        files: fixture.files.len(),
        glob_matches: count_queue_glob_matches(
            &fixture.root,
            &fixture.files,
            &fixture.queue_enqueues,
            &fixture.queue_workers,
        ),
        ..LanguageFrontendSummary::default()
    }
}

fn fact_len(
    maps: [&LangFactMap; 8],
    field: impl Fn(&crate::codebase::lang_frontends::LangFileFacts) -> usize,
) -> usize {
    maps.iter()
        .flat_map(|map| map.files.values())
        .map(field)
        .sum()
}

#[cfg(test)]
#[path = "language_frontends/tests.rs"]
mod tests;
