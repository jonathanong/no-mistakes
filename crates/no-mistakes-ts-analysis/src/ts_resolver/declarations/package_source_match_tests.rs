use super::matches_runtime_export;
use serde_json::json;

#[test]
fn star_target_matches_the_substituted_source_file() {
    let exports = json!({ ".": "./index.ts", "./*": "./*.ts" });
    assert!(matches_runtime_export(&exports, "shape.ts"));
    assert!(matches_runtime_export(&exports, "theme/color.ts"));
    assert!(matches_runtime_export(&exports, "index.ts"));
    assert!(!matches_runtime_export(&exports, "shape.js"));
    assert!(!matches_runtime_export(&exports, "theme.tsx"));
    assert!(!matches_runtime_export(&exports, "../shape.ts"));
}

#[test]
fn import_condition_ignores_the_require_star_target() {
    let exports = json!({ "./*": { "import": "./*.ts", "require": "./*.js" } });
    assert!(matches_runtime_export(&exports, "shape.ts"));
    assert!(!matches_runtime_export(&exports, "shape.js"));
}

#[test]
fn a_star_export_ignores_a_non_string_target() {
    let exports = json!({ "./*": ["./*.ts"] });
    assert!(!matches_runtime_export(&exports, "shape.ts"));
    let nested = json!({ "./*": { "import": null } });
    assert!(!matches_runtime_export(&nested, "shape.ts"));
}

#[test]
fn top_level_condition_targets_name_the_published_file() {
    let exports = json!({ "import": "./index.ts", "default": "./index.ts" });
    assert!(matches_runtime_export(&exports, "index.ts"));
    assert!(!matches_runtime_export(&exports, "styles.css"));
    let nested = json!({ "import": { "default": "./src/index.ts" } });
    assert!(matches_runtime_export(&nested, "src/index.ts"));
}

#[test]
fn exact_subpath_and_nested_star_use_the_runtime_target() {
    let exact = json!({ ".": "./index.ts", "./badge.ts": "./badge.ts" });
    assert!(matches_runtime_export(&exact, "badge.ts"));
    assert!(!matches_runtime_export(&exact, "missing.ts"));

    let nested = json!({ "./src/*": "./src/*.ts" });
    assert!(matches_runtime_export(&nested, "src/shape.ts"));
    assert!(!matches_runtime_export(&nested, "shape.ts"));
}
