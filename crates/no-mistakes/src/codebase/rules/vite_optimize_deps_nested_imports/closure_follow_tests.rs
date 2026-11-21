use super::closure::{is_script_module, runtime_imports};
use crate::codebase::rules::source_store_for_files;
use crate::codebase::ts_resolver::{self, TsConfig};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn closure_packages(scenario: &str) -> BTreeSet<(String, String)> {
    let root = ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vite-optimize-deps-nested-imports/fixture")
            .join(scenario),
    );
    let dir = root.join("packages/widgets");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("package.json")).expect("package.json"),
    )
    .expect("package.json");
    let sources = source_store_for_files(&[]);
    let imports = runtime_imports(
        &root,
        &dir,
        &manifest,
        "widgets",
        &sources,
        &[],
        &BTreeSet::new(),
    )
    .expect("published entry stays analyzable");
    imports
        .into_iter()
        .map(|import| (import.package, import.file))
        .collect()
}

#[test]
fn closure_skips_asset_targets_and_follows_dot_specifiers() {
    assert_eq!(
        closure_packages("asset-imports"),
        BTreeSet::from([
            ("items".to_string(), "packages/widgets/index.ts".to_string()),
            (
                "labels".to_string(),
                "packages/widgets/src/deep/index.ts".to_string()
            ),
            (
                "orders".to_string(),
                "packages/widgets/src/index.ts".to_string()
            ),
        ])
    );
}

#[test]
fn closure_uses_a_nested_export_condition_instead_of_an_unbuilt_main() {
    assert_eq!(
        closure_packages("nested-conditions"),
        BTreeSet::from([(
            "items".to_string(),
            "packages/widgets/src/index.ts".to_string()
        )])
    );
}

#[test]
fn closure_skips_dependency_directories_and_imports_outside_the_package() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vite-optimize-deps-nested-imports/fixture/closure-edges");
    let dir = root.join("packages/widgets");
    assert!(root.join("outside.ts").is_file());
    let sources = source_store_for_files(&[]);
    let manifest = serde_json::json!({
        "name": "widgets",
        "exports": {
            ".": "./index.ts",
            "./*": "./*.ts"
        }
    });
    let entry = dir.join("index.ts");
    let resolved = ts_resolver::resolve_import("../../outside.ts", &entry, &TsConfig::default())
        .expect("outside import resolves");
    assert!(!ts_resolver::normalize_path(&resolved).starts_with(&dir));
    let imports = runtime_imports(
        &root,
        &dir,
        &manifest,
        "widgets",
        &sources,
        &[
            dir.join("node_modules/items/index.ts"),
            PathBuf::from("packages/widgets/index.ts"),
        ],
        &BTreeSet::new(),
    )
    .unwrap();
    assert!(
        imports.iter().all(|import| import.specifier != "orders"),
        "a relative import that resolves outside the package is not followed"
    );
    assert!(imports.is_empty());
    let skipped = runtime_imports(
        &root,
        &dir,
        &serde_json::json!({
            "name": "widgets",
            "main": "node_modules/items/index.ts"
        }),
        "widgets",
        &sources,
        &[],
        &BTreeSet::new(),
    )
    .unwrap();
    assert!(skipped.is_empty());
}

#[test]
fn script_modules_are_javascript_and_typescript_extensions_only() {
    for extension in ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"] {
        assert!(is_script_module(Path::new(&format!("index.{extension}"))));
    }
    assert!(!is_script_module(Path::new("styles.css")));
    assert!(!is_script_module(Path::new("data.json")));
    assert!(!is_script_module(Path::new("mark.svg")));
    assert!(!is_script_module(Path::new("app.wasm")));
    assert!(!is_script_module(Path::new("notes")));
}
