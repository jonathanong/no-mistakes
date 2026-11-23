use super::check_json_impl;
use serde_json::{json, Value};
use std::path::PathBuf;

fn check(config: &str, case: &str) -> Value {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pinned-hash");
    let root = base.join("fixture/cases").join(case);
    let config = base.join("configs").join(config);
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "config": config}).to_string(),
    ))
    .unwrap();
    serde_json::from_str(&output).unwrap()
}

#[test]
fn check_json_images_option_preserves_default_and_routes_expressions() {
    let off = check("images-off.yml", "images");
    let old_action = off["rules"].as_array().unwrap();
    assert_eq!(old_action.len(), 1, "{off}");
    assert_eq!(old_action[0]["line"], 28);
    assert_eq!(off["advisories"], json!([]));

    let on = check("images-on.yml", "images");
    let rules = on["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 12, "{on}");
    assert!(rules.contains(&old_action[0]));
    assert_eq!(on["advisories"], json!([]));
    assert!(rules.iter().any(|finding| {
        finding["file"] == ".github/workflows/images.yml"
            && finding["line"] == 16
            && finding["message"].as_str().unwrap().contains("postgres:16")
    }));
    assert!(rules.iter().any(|finding| {
        finding["file"] == ".github/actions/example/action.yml" && finding["line"] == 5
    }));
    let suppressed = check("images-on.yml", "image-suppression");
    assert_eq!(suppressed["rules"], json!([]));
    assert_eq!(suppressed["advisories"], json!([]));

    let ignored = check("images-on.yml", "image-expressions");
    assert_eq!(ignored["rules"], json!([]));
    assert_eq!(ignored["advisories"], json!([]));
    let report = check("images-report.yml", "image-expressions");
    assert_eq!(report["rules"].as_array().unwrap().len(), 4, "{report}");
    assert_eq!(report["advisories"], json!([]));
    let advise = check("images-advise.yml", "image-expressions");
    assert_eq!(advise["rules"], json!([]));
    assert_eq!(
        advise["advisories"].as_array().unwrap().len(),
        4,
        "{advise}"
    );
}
