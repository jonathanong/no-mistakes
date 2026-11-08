#[test]
fn prepared_postgres_catalog_bounds_match_standalone() {
    let fixture = category_fixture("rules", "postgres-bounded-statements");
    for (directory, config, expected) in [
        ("catalog-resolution", "key-resolution.yml", 5),
        ("catalog-resolution", "key-operators.yml", 2),
        ("catalog-resolution", "key-booleans.yml", 1),
        ("catalog-resolution", "view-bounds.yml", 1),
        ("catalog-resolution", "view-allow.yml", 0),
        ("catalog-resolution", "view-suppression.yml", 0),
        ("catalog-resolution-identity", "view-identity.yml", 1),
        ("schemaless-key-resolution", "qualified-bare-tables.yml", 3),
        ("column-policy", "../../configs/column-bounds.yml", 6),
        ("../materialized-bodies/probes", "off.yml", 0),
        ("../materialized-bodies/probes", "advise.yml", 0),
        ("../materialized-bodies/probes", "report.yml", 17),
        (
            "column-policy",
            "../../configs/column-bounds-composite-prefix.yml",
            1,
        ),
        ("column-policy", "../../configs/recursive-depth.yml", 10),
        ("join-roots", "valid.yml", 22),
        ("fixedpoint", "valid.yml", 27),
        ("issue-3037", "valid.yml", 3),
        ("issue-3037", "cte-valid.yml", 0),
        ("root-proofs", "valid.yml", 18),
        ("root-proofs", "invalid.yml", 19),
        ("../statement-groups/fixture", "valid.yml", 7),
        ("../statement-groups/fixture", "controls.yml", 6),
        ("../statement-groups/fixture", "embedded.yml", 4),
        ("../statement-groups/fixture", "embedded-with-extra.yml", 4),
        ("../statement-groups/fixture", "variant-prefix.yml", 1),
        ("../statement-groups/fixture", "repeated-helper.yml", 2),
        ("../variants/correlated-append", ".no-mistakes.yml", 1),
        ("../variants/correlated-append", "undeclared.yml", 2),
        ("../variants/correlated-append", "advise.yml", 0),
    ] {
        let root = fixture.join(directory);
        let standalone = parse_json(
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config, "includeSqlVerdicts": true}),
            ))
            .unwrap(),
        );
        let aggregate = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": config,
                "reports": [{"type": "check", "includeSqlVerdicts": true}]
            })))
            .unwrap(),
        );
        assert_eq!(
            standalone["rules"].as_array().unwrap().len(),
            expected,
            "{config}"
        );
        assert_eq!(aggregate["reports"][0]["result"], standalone, "{config}");
    }
}
