use super::*;

#[test]
fn unknown_global_guards_do_not_publish_parameter_array_proofs() {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("3304-unknown-guards.mjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = read_fixture(
        "test-no-live-repository-scan/facts",
        "3304-unknown-guards.mjs",
        true,
    );
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let provenance = facts.path_provenance.as_ref().unwrap();
    let offset = |case: &str| {
        let line = source
            .lines()
            .find(|line| line.contains(&format!("'{case}'")))
            .unwrap();
        (source.find(line).unwrap() + line.find("readdirSync(").unwrap()) as u32
    };
    for case in [
        "global-condition",
        "global-array",
        "global-alias",
        "global-rest",
    ] {
        assert!(
            provenance
                .call_guards
                .get(&offset(case))
                .is_none_or(Vec::is_empty),
            "{case}: {:?}",
            provenance.call_guards
        );
    }
    let guards = &provenance.call_guards[&offset("parameter")];
    assert_eq!(guards.len(), 1);
    assert_eq!(guards[0].parameter, 0);
    assert_eq!(guards[0].index, 0);
    assert!(!guards[0].length);
    assert_eq!(guards[0].expected, serde_json::json!("scan"));
    assert_eq!(
        guards[0].owner,
        source.find("function parameter(").unwrap() as u32
    );
}
