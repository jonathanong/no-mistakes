use super::analyze_file;
use std::path::PathBuf;

fn fixture(category: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(name)
        .join("fixture")
}

#[test]
fn analyze_basic_greeting() {
    let root = fixture("react-traits-components", "basic");
    let file = root.join("app/components/Greeting.tsx");
    let result = analyze_file(&file, &root).expect("should succeed");
    assert_eq!(result.components.len(), 1);
    assert_eq!(result.components[0].name, "default");
}

#[test]
fn analyze_counter_has_state() {
    let root = fixture("react-traits-components", "basic");
    let file = root.join("app/components/Counter.tsx");
    let result = analyze_file(&file, &root).expect("should succeed");
    assert!(result.components[0].has_state);
}

#[test]
fn nonexistent_file_returns_error() {
    let root = fixture("react-traits-components", "basic");
    let file = root.join("app/components/DoesNotExist.tsx");
    assert!(analyze_file(&file, &root).is_err());
}

#[test]
fn invalid_tsx_returns_error() {
    let root = fixture("react-traits-analyze", "file-error");
    let file = root.join("invalid.tsx");
    assert!(analyze_file(&file, &root).is_err());
}

#[test]
fn analyze_server_component_environment() {
    use crate::react_traits::report::types::Environment;
    let root = fixture("react-traits-analyze", "environments");
    let file = root.join("ServerComp.tsx");
    let result = analyze_file(&file, &root).expect("should succeed");
    assert_eq!(result.components[0].environment, Environment::Server);
}

#[test]
fn analyze_client_component_environment() {
    use crate::react_traits::report::types::Environment;
    let root = fixture("react-traits-analyze", "environments");
    let file = root.join("ClientComp.tsx");
    let result = analyze_file(&file, &root).expect("should succeed");
    assert_eq!(result.components[0].environment, Environment::Client);
}

#[test]
fn multi_component_scopes_fetch_to_component_span() {
    // Two components in one file: FetchingComponent has fetch, PureComponent does not.
    // The FetchVisitor's in_scope = false path is exercised for calls outside each span.
    let root = fixture("react-traits-analyze", "multi-component");
    let file = root.join("app/components/Mixed.tsx");
    let analysis = analyze_file(&file, &root).expect("should analyze");

    let fetching = analysis
        .components
        .iter()
        .find(|c| c.name == "FetchingComponent")
        .expect("FetchingComponent not found");
    let pure = analysis
        .components
        .iter()
        .find(|c| c.name == "PureComponent")
        .expect("PureComponent not found");

    assert!(
        !fetching.fetches.is_empty(),
        "FetchingComponent should detect fetch"
    );
    assert!(
        pure.fetches.is_empty(),
        "PureComponent should not inherit FetchingComponent's fetch"
    );
}

#[test]
fn analyze_file_from_visible_reuses_prepared_source_store() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;

    let root = fixture("react-traits-components", "basic");
    let file = root.join("app/components/Greeting.tsx");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&file)));
    let store = SourceStore::new(inventory);
    let visible: crate::fx::PathSet = [crate::codebase::ts_resolver::normalize_path(&file)]
        .into_iter()
        .collect();

    let first = super::analyze_file_from_visible(&file, &root, &visible, Some(&store))
        .expect("should succeed");
    let reads = store.physical_read_count();
    assert!(
        reads >= 1,
        "react file analysis must read through the prepared SourceStore"
    );

    let second = super::analyze_file_from_visible(&file, &root, &visible, Some(&store))
        .expect("should succeed");
    assert_eq!(first.components[0].name, "default");
    assert_eq!(second.components[0].name, "default");
    assert_eq!(
        store.physical_read_count(),
        reads,
        "react file analysis must reuse the prepared SourceStore"
    );
}

fn analyze_program_inner_source() -> &'static str {
    let source = include_str!("../file.rs");
    let start = source
        .find("fn analyze_program_inner(")
        .expect("analyze_program_inner must exist");
    let after = &source[start..];
    let header = "fn analyze_program_inner(";
    match after[header.len()..].find("\nfn ") {
        Some(rel) => &after[..header.len() + rel],
        None => after,
    }
}

#[test]
fn analyze_program_inner_fuses_per_component_trait_walks() {
    let inner = analyze_program_inner_source();
    assert_eq!(
        inner.matches("collect_file_trait_hits").count(),
        1,
        "analyze_program_inner must invoke collect_file_trait_hits once"
    );
    assert!(
        !inner.contains("detect_has_state")
            && !inner.contains("detect_props")
            && !inner.contains("detect_uses_memo")
            && !inner.contains("detect_context_provider")
            && !inner.contains("detect_uses_suspense")
            && !inner.contains("collect_jsx_children"),
        "per-component trait detectors must not walk the program again"
    );
}
