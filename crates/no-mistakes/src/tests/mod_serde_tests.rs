use super::*;

#[test]
fn impact_reason_resource_details_round_trip_and_legacy_details_are_optional() {
    let reason = ImpactReason {
        changed_file: "resources/schema.sql".to_string(),
        path: vec![
            "resources/schema.sql".to_string(),
            "src/load.ts".to_string(),
            "src/load.test.ts".to_string(),
        ],
        via: vec!["resource".to_string(), "dependency".to_string()],
        via_details: vec![
            Some(ImpactEdgeDetail::Resource {
                consumer_file: "src/load.ts".to_string(),
                call_sites: vec![ResourceCallSite {
                    call_kind: "read-file".to_string(),
                    line: 8,
                }],
            }),
            None,
        ],
    };
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("via_details"));
    assert_eq!(serde_json::from_str::<ImpactReason>(&json).unwrap(), reason);

    let legacy = r#"{"changed_file":"a.ts","path":["a.ts","a.test.ts"],"via":["dependency"]}"#;
    let legacy_reason: ImpactReason = serde_json::from_str(legacy).unwrap();
    assert!(legacy_reason.via_details.is_empty());
    assert!(
        !serde_json::to_string(&legacy_reason)
            .unwrap()
            .contains("via_details")
    );
}
