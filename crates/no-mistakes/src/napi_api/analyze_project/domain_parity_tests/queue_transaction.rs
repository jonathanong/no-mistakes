#[test]
fn queue_transaction_rule_is_shared_by_standalone_and_batched_checks() {
    let root = category_fixture("rules", "queue-no-enqueue-in-transaction").join("behavior");
    let standalone = parse_json(crate::napi_api::check_json_impl(json!({ "root": root })).unwrap());
    let findings = standalone["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{standalone:#?}");
    assert_eq!(findings[0]["rule"], "queue-no-enqueue-in-transaction");
    assert_eq!(findings[0]["target"], "@sample/jobs#enqueue");
    assert_eq!(findings[1]["target"], "@sample/jobs#enqueue");

    let options = json!({ "root": root, "reports": [{ "type": "check" }] });
    let batch = analyze_project_json_impl(options).unwrap();
    assert_eq!(report_results(&batch), [standalone]);
}

#[test]
fn scoped_queue_transaction_advisories_match_in_batched_checks() {
    let root = category_fixture("rules", "queue-no-enqueue-in-transaction").join("scoped-handles");
    let config = root.join("advise.yml");
    let standalone = parse_json(
        crate::napi_api::check_json_impl(json!({ "root": root, "config": config })).unwrap(),
    );
    assert_eq!(
        standalone["rules"].as_array().unwrap().len(),
        5,
        "{standalone:#}"
    );
    assert_eq!(
        standalone["advisories"].as_array().unwrap().len(),
        14,
        "{standalone:#}"
    );
    let batch = analyze_project_json_impl(json!({
        "root": root, "reports": [{ "type": "check", "config": config }]
    }))
    .unwrap();
    assert_eq!(report_results(&batch), [standalone]);
}
