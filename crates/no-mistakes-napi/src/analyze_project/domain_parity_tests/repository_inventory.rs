#[test]
fn prepared_check_keeps_repository_inventory_below_source_skips() {
    let fixture = no_mistakes::test_support::materialize_gitignore_fixture(
        "banned-paths-source-skips",
    );
    no_mistakes::test_support::git_init(fixture.path());
    no_mistakes::test_support::git_add_all(fixture.path());
    let root = fixture.path();
    let standalone =
        parse_json(crate::check_json_impl(crate::options::test_json_arg(json!({ "root": root }).to_string())).unwrap());
    let aggregate = parse_json(
        analyze_project_json_impl(
            crate::options::test_json_arg(json!({
                "root": root,
                "reports": [{ "type": "check", "id": "check" }]
            })
            .to_string(),)
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
