use super::{artifact_value, artifact_value_observed, AxisRegexEvent};
use crate::codebase::workflow_topology::value_primitives::to_json;

fn fixture() -> serde_yaml::Value {
    let path = crate::codebase::workflow_topology::tests::fixture("artifact-axis-lifetime")
        .join("requests.yml");
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn dollar_axis_regexes_are_compiled_once_per_axis_and_released_per_request() {
    let saved = fixture();
    let mut reports = Vec::new();
    for request in saved["requests"].as_sequence().unwrap() {
        let matrix = to_json(&request["matrix"]);
        let raw = request["raw"].as_str().unwrap();
        let expected_compilations = request["compiled_axes"].as_u64().unwrap() as usize;
        let mut events = Vec::new();
        let report = artifact_value_observed(raw, Some(&matrix), |event| events.push(event));
        // The second identical request must compile again; a global cache's
        // warmed entry would skip the actual compilation event and fail here.
        assert_eq!(
            events,
            [AxisRegexEvent::Compiled, AxisRegexEvent::Released].repeat(expected_compilations)
        );
        assert_eq!(
            artifact_value(raw, Some(&matrix)),
            report,
            "public entry parity"
        );
        let actual = serde_yaml::to_value(&report).unwrap();
        assert_eq!(
            actual, request["expected"],
            "complete artifact values and multiplicity"
        );
        reports.push(report);
    }
    assert_eq!(
        reports[0], reports[1],
        "fresh requests preserve full output"
    );
}
