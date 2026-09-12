use super::*;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn apply_own_rejects_invalid_compiler_and_list_shapes() {
    let root = PathBuf::from("/repo");
    let path = root.join("tsconfig.json");
    let mut config = EffectiveConfig::new(path.clone(), root.clone());
    let ok = |value: &str| Ok(root.join(value));

    let cases = [
        (
            json!({ "compilerOptions": { "moduleResolution": false } }),
            "moduleResolution",
        ),
        (json!({ "compilerOptions": { "baseUrl": 1 } }), "baseUrl"),
        (json!({ "compilerOptions": { "outDir": false } }), "outDir"),
        (json!({ "compilerOptions": { "paths": [] } }), "paths"),
        (
            json!({ "compilerOptions": { "paths": { "@lib/*": "src" } } }),
            "paths",
        ),
        (json!({ "files": "src/entry.ts" }), "files"),
        (json!({ "files": [1] }), "files"),
        (json!({ "include": "src/**" }), "include"),
        (json!({ "exclude": false }), "exclude"),
        (json!({ "references": { "path": "pkg" } }), "references"),
        (json!({ "references": [1] }), "references"),
        (json!({ "references": [{}] }), "references"),
    ];
    for (value, expected) in cases {
        let err = config
            .apply_own(&value, &path, &root, ok)
            .expect_err("invalid config should fail");
        assert!(
            err.to_ascii_lowercase()
                .contains(&expected.to_ascii_lowercase()),
            "{expected}: {err}"
        );
    }

    let err = config
        .apply_own(&json!({ "references": ["pkg"] }), &path, &root, |_| {
            Err("missing project reference".to_string())
        })
        .expect_err("unresolved project references should fail");
    assert!(err.contains("missing project reference"));
}
