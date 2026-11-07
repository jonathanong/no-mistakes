use super::*;

#[test]
fn appended_view_probe_matches_literal_through_public_check_and_project() {
    let root = crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates");
    for shape in ["append", "object", "literal"] {
        for status in ["pass", "positive"] {
            let name = format!("append-view-{shape}-{status}");
            let config = root
                .parent()
                .unwrap()
                .join("configs")
                .join(format!("{name}-max.yml"));
            let check: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
                serde_json::from_str(
                    &check_json_impl(crate::napi_api::options::test_json_arg(
                        json!({"root": root, "config": config}).to_string(),
                    ))
                    .unwrap(),
                )
                .unwrap(),
            );
            assert_eq!(check["warnings"], json!([]), "{name}: {check}");
            assert_eq!(check["advisories"], json!([]), "{name}: {check}");
            let findings = check["rules"].as_array().unwrap();
            assert_eq!(
                findings.len(),
                2 * usize::from(status == "positive"),
                "{name}: {check}"
            );
            for finding in findings {
                assert_eq!(finding["rule"], "postgres-no-transformed-uuid-predicates");
                assert_eq!(finding["file"], format!("{name}.ts"));
                assert!([json!("view_posts.id"), json!("view_posts.root_id")]
                    .contains(&finding["target"]));
                assert_eq!(finding["line"], if shape == "literal" { 5 } else { 6 });
            }
            let project: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
                serde_json::from_str(
                    &crate::napi_api::analyze_project::analyze_project_json_impl(
                        crate::napi_api::options::test_json_arg(
                            json!({
                                "root": root,
                                "config": config,
                                "reports": [{"type": "check"}],
                            })
                            .to_string(),
                        ),
                    )
                    .unwrap(),
                )
                .unwrap(),
            );
            assert_eq!(project["reports"][0]["result"], check, "{name}");
        }
    }
}
