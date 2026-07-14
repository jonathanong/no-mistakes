use super::fixture;
use crate::codebase::ts_resolver::ImportResolver;
use crate::queue::extract_helpers::quoted_prefix;
use crate::queue::extract_model::FileFacts;
use crate::queue::resolver::{load_tsconfig_from_visible, TsConfig};
use crate::queue::source::discover_source_files;
use std::path::PathBuf;

fn load_tsconfig(
    root: &std::path::Path,
    explicit: Option<&std::path::Path>,
) -> anyhow::Result<TsConfig> {
    let visible = crate::codebase::ts_source::discover_visible_paths(root);
    load_tsconfig_from_visible(root, explicit, &visible)
}

fn resolve_import_inner(
    specifier: &str,
    current_file: &std::path::Path,
    root: &std::path::Path,
    tsconfig: &TsConfig,
) -> Option<PathBuf> {
    ImportResolver::new(tsconfig)
        .with_queue_compatibility(root)
        .resolve(specifier, current_file)
}

fn resolve_import(
    specifier: &str,
    current_file: &std::path::Path,
    root: &std::path::Path,
    tsconfig: &TsConfig,
) -> Option<PathBuf> {
    resolve_import_inner(specifier, current_file, root, tsconfig)
}

pub(super) fn extract_file_with_factories(
    path: &std::path::Path,
    factory_names: &[String],
) -> anyhow::Result<FileFacts> {
    let source = std::fs::read_to_string(path)?;
    crate::ast::with_program(path, &source, |program, _| {
        crate::queue::extract::extract_program_with_factories(path, &source, program, factory_names)
    })
}
#[test]
fn extract_file_records_import_forms_without_queue_semantics() {
    let root = fixture("syntax");
    let facts = extract_file_with_factories(&root.join("imports.ts"), &[]).unwrap();
    assert!(facts
        .imports
        .iter()
        .any(|import| import.imported == "default" && import.local == "Bull"));
    assert!(facts
        .imports
        .iter()
        .any(|import| import.imported == "legacyName" && import.local == "legacy"));
}

#[test]
fn resolver_handles_exact_paths_base_url_and_indexes() {
    let root = fixture("resolver");
    let tsconfig = load_tsconfig(&root, Some(&root.join("tsconfig.json"))).unwrap();
    let current = root.join("src/enqueue.ts");
    assert_eq!(
        resolve_import("@queues", &current, &root, &tsconfig),
        Some(root.join("src/queues/index.ts").canonicalize().unwrap())
    );
    assert_eq!(
        resolve_import("src/processors/worker", &current, &root, &tsconfig),
        Some(
            root.join("src/processors/worker.ts")
                .canonicalize()
                .unwrap()
        )
    );
    assert_eq!(
        resolve_import("@queue-dir", &current, &root, &tsconfig),
        Some(crate::codebase::ts_resolver::normalize_path(
            &root.join("src/queues/index.ts"),
        ))
    );
    assert_eq!(
        resolve_import("./direct.ts", &current, &root, &tsconfig),
        Some(root.join("src/direct.ts").canonicalize().unwrap())
    );
}

#[test]
fn resolver_accepts_jsonc_tsconfig() {
    let root = fixture("resolver");
    let tsconfig = load_tsconfig(&root, Some(&root.join("tsconfig-jsonc.json"))).unwrap();
    let current = root.join("src/enqueue.ts");
    assert_eq!(
        resolve_import("@queues", &current, &root, &tsconfig),
        Some(root.join("src/queues/index.ts").canonicalize().unwrap())
    );
}

#[test]
fn quoted_prefix_returns_partial_value_when_closing_quote_is_absent() {
    assert_eq!(quoted_prefix("\"partial"), Some("partial".to_string()));
}

#[test]
fn resolver_handles_relative_tsconfig_and_fallback_targets() {
    let root = fixture("resolver");
    let tsconfig = load_tsconfig(&root, Some(std::path::Path::new("tsconfig.json"))).unwrap();
    let current = root.join("src/enqueue.ts");
    assert_eq!(
        resolve_import("@fallback/worker", &current, &root, &tsconfig),
        Some(
            root.join("src/processors/worker.ts")
                .canonicalize()
                .unwrap()
        )
    );
}

#[test]
fn resolver_compatibility_preserves_configured_alias_order() {
    let root = fixture("resolver");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        base_url: None,
        // The broader alias is intentionally first. Standalone queue analysis
        // has historically treated configuration order as authoritative.
        paths: vec![
            ("@*".to_string(), vec!["src/*".to_string()]),
            (
                "@queues".to_string(),
                vec!["src/processors/worker".to_string()],
            ),
        ],
    };
    assert_eq!(
        resolve_import("@queues", &root.join("src/enqueue.ts"), &root, &tsconfig),
        Some(crate::codebase::ts_resolver::normalize_path(
            &root.join("src/queues/index.ts"),
        ))
    );
}

#[test]
fn resolver_defaults_when_no_tsconfig_exists() {
    let config = load_tsconfig(&fixture("basic"), None).unwrap();
    assert!(config.base_url.is_none());
    assert!(config.paths.is_empty());
}

#[test]
fn invalid_tsconfig_returns_parse_error() {
    let root = fixture("invalid-tsconfig");
    let err = load_tsconfig(&root, None).unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn discovery_skips_dependency_and_build_directories() {
    let root = fixture("syntax");
    let files = discover_source_files(&root);
    for skipped in ["node_modules", "target", "build"] {
        assert!(
            !files
                .iter()
                .any(|file| file.to_string_lossy().contains(skipped)),
            "discovered file under {skipped}"
        );
    }
}

#[test]
fn queue_compatibility_appends_extensions_to_dotted_import_stems() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queue/dotted-worker");
    let config = load_tsconfig(&root, None).unwrap();
    let importer = root.join("src/enqueue.ts");
    // `.queue` is part of the stem; replacing it would incorrectly probe `worker.ts`.
    assert_eq!(
        resolve_import("./worker.queue", &importer, &root, &config),
        Some(crate::codebase::ts_resolver::normalize_path(
            &root.join("src/worker.queue.ts"),
        )),
    );
    // Recognized emitted extensions retain the historical replacement fallback.
    assert_eq!(
        resolve_import("./worker.js", &importer, &root, &config),
        Some(crate::codebase::ts_resolver::normalize_path(
            &root.join("src/worker.ts"),
        )),
    );
}
