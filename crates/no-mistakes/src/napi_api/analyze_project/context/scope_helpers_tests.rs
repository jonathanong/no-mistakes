use super::*;

#[test]
fn string_option_rejects_non_string_values() {
    let request: AnalyzeReportRequest = serde_json::from_value(serde_json::json!({
        "type": "symbols",
        "root": 1
    }))
    .unwrap();

    assert_eq!(
        string_option(&request, "root").unwrap_err().to_string(),
        "root must be a string"
    );
}

#[test]
fn effective_path_handles_relative_absolute_and_missing_values() {
    let root = Path::new("/repo");
    assert_eq!(
        effective_path(root, Some("src/../app.ts")),
        Some(PathBuf::from("/repo/app.ts"))
    );
    assert_eq!(
        effective_path(root, Some("/other/app.ts")),
        Some(PathBuf::from("/other/app.ts"))
    );
    assert_eq!(effective_path(root, None), None);
}
