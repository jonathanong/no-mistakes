#[test]
fn prepared_check_keeps_repository_inventory_below_source_skips() {
    let fixture = crate::test_support::materialize_gitignore_fixture(
        "banned-paths-source-skips",
    );
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = fixture.path();
    let standalone =
        parse_json(crate::napi_api::check_json_impl(json!({ "root": root }).to_string()).unwrap());
    let aggregate = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{ "type": "check", "id": "check" }]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let prepared = &aggregate["reports"][0]["result"];
    let files = prepared["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "banned-paths")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(prepared, &standalone);
    assert_eq!(
        files,
        vec![
            "build/blocked.patch",
            "dist/blocked.patch",
            "fixtures/blocked.patch",
            "nested/blocked.patch",
            "target/blocked.patch",
        ]
    );
}
