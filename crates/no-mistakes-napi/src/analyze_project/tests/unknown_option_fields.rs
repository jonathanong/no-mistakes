use super::options;
use super::options_test_support::parse_options;
use super::types::AnalyzeProjectOptions;
use serde_json::json;

#[test]
fn report_option_helpers_reject_unknown_fields() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "reports": [
                { "type": "symbols", "files": ["a.mts"], "notAField": true },
                { "type": "importUsages", "notAField": true },
                { "type": "queues", "unknownQueue": true },
                { "type": "flow", "bogus": 1 },
                { "type": "playwrightCheck", "nope": true },
                { "type": "effects", "kind": 1 },
                { "type": "rsc-callers", "component": false },
                { "type": "dependencies", "depth": "nope" }
            ]
        })
        .to_string(),
    )
    .unwrap();
    assert!(options::symbols_options(&options.reports[0], &options).is_err());
    assert!(options::import_usages_options(&options.reports[1], &options).is_err());
    assert!(options::project_options(&options.reports[2], &options).is_err());
    assert!(options::flow_options(&options.reports[3], &options).is_err());
    assert!(options::playwright_options(&options.reports[4], &options).is_err());
    assert!(options::effects_options(&options.reports[5], &options).is_err());
    assert!(options::rsc_callers_options(&options.reports[6], &options).is_err());
    assert!(options::traverse_options(&options.reports[7], &options).is_err());
}
