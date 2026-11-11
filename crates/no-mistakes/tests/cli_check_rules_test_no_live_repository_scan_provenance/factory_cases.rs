use super::{check_in, enumeration_locations};

#[test]
fn factory_edge_cases_keep_node_identity_conservative() {
    let (_, report) = check_in("temp-factory-identity", ".no-mistakes.yml");
    let mut enumerations = enumeration_locations(&report["rules"]);
    enumerations.extend(enumeration_locations(&report["advisories"]));
    for (file, line) in [
        ("deep-selection.test.cts", 7),
        ("computed-call.test.cts", 5),
        ("direct-promises-escape.test.cts", 10),
        ("direct-promises-write.test.cts", 7),
        ("literal-write.test.cts", 7),
        ("reassigned-view.test.cts", 8),
        ("reassigned-view.test.cts", 9),
        ("promises-replaced.test.cts", 7),
        ("promises-literal-write.test.cts", 7),
        ("empty-call.test.cts", 7),
        ("other-method.test.cts", 8),
        ("second-argument.test.cts", 8),
        ("nested-spy.test.cts", 9),
    ] {
        let expected = format!("tests/{file}:{line}");
        assert!(
            enumerations.contains(&expected),
            "{expected}: {enumerations:?}"
        );
    }
    assert!(
        !enumerations.contains(&"tests/mixed-import.test.mts:9".to_string()),
        "{enumerations:?}"
    );
}
