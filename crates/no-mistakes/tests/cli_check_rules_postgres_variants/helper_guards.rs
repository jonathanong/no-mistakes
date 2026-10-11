use super::*;

#[test]
fn unsummarized_helper_guards_fail_closed_without_stale_finite_offset_findings() {
    let ignored = check("postgres-no-offset", "helper-guard-ignore");
    let findings = ignored["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{ignored}");
    assert_eq!(findings[0]["line"], 10, "{ignored}");
    assert!(
        findings[0]["target"]
            .as_str()
            .unwrap()
            .starts_with("offset"),
        "{ignored}"
    );
    let failed = check("postgres-no-offset", "helper-guard-fail");
    let findings = failed["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 6, "{failed}");
    let opaque = findings
        .iter()
        .filter(|finding| finding["target"] == "unanalyzable")
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(opaque, [11, 12, 13, 19, 26], "{failed}");
}
