use super::*;

#[test]
fn named_client_configuration_reaches_defaults_and_arguments_of_factory_clients() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("named-clients.yml")), None)
            .unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/local-client-entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 5],
        "{findings:#?}"
    );
    let report = crate::effects_query::run(
        &root,
        Some(&root.join("named-clients.yml")),
        None,
        "valkey",
        Path::new("src/local-client-entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.callee.as_str())
            .collect::<Vec<_>>(),
        ["primary.get", "session.info", "session.scan"]
    );
}
