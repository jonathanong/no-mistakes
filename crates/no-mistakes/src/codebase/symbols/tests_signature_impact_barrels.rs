#[test]
fn signature_impact_accepts_star_barrel_concrete_symbols() {
    let out = run_capture(impact_file_args(
        "star-date-barrel.mts",
        "parseDate",
        Format::Json,
    ));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["definition"]["file"], "star-date-barrel.mts");
    assert_eq!(v["definition"]["symbol"], "parseDate");
}

#[test]
fn signature_impact_rejects_symbols_not_exported_by_star_barrel() {
    let err = impact::collect_report(&impact_file_args(
        "star-date-barrel.mts",
        "default",
        Format::Json,
    ))
    .unwrap_err();
    assert!(err.to_string().contains("is not exported"));
}

#[test]
fn signature_impact_treats_local_import_export_barrels_as_exports() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "local-date-barrel.mts" && entry["symbol"] == "parseDate"
    }));
    assert!(!v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "local-date-barrel.mts" && entry["symbol"] == "parseDate"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "local-barrel-consumer.mts" && entry["symbol"] == "parseLocalDate"
    }));
}

#[test]
fn signature_impact_keeps_same_name_wrappers_as_callers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(!v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "same-name-wrapper.mts" && entry["symbol"] == "parseDate"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "same-name-wrapper.mts" && entry["symbol"] == "parseDate"
    }));
}

#[test]
fn signature_impact_passes_explicit_config_to_graph() {
    let mut args = impact_args("parseDate", Format::Json);
    args.config = Some(PathBuf::from("exclude-other-test.no-mistakes.yml"));
    let out = run_capture(args);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(!v["suggestedTests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "other.test.mts"));
}

#[test]
fn signature_impact_pipeline_run_handles_signature_impact_mode() {
    run(impact_args("parseDate", Format::Json)).unwrap();
    let mut timed = impact_args("parseDate", Format::Json);
    timed.timings = true;
    run(timed).unwrap();
}
