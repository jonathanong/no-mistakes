use super::*;

#[test]
fn saved_workflow_covers_optional_wrapper_arguments_and_port_variables() {
    let (root, config, files) = load_fixture("3304-wrappers");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    let ci_findings: Vec<_> = findings
        .iter()
        .filter(|finding| finding.file.ends_with("ci.yml"))
        .collect();
    let finding = ci_findings
        .iter()
        .find(|finding| finding.message.contains("in `15432:5432`"))
        .expect("literal timeout mapping");
    assert!(finding.message.contains("in `15432:5432`"), "{finding:?}");
    assert_eq!(finding.line, 9, "{finding:?}");
    let source = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    assert_eq!(
        nth_line(&source, finding.line),
        "      - run: timeout -k 5s 30s docker run --publish 15432:5432 postgres"
    );
    assert!(source.contains("env -C /tmp docker run -p 1:2 postgres"));
    assert!(source.contains("env -u PORT docker run -p 1:2 postgres"));
    assert!(source.contains("${PORT}:5432"));
    assert!(source.contains("${{ vars.HOST_PORT }}:5432"));
}

#[test]
fn saved_workflow_wrappers_keep_literal_ports_visible_and_dynamic_hosts_silent() {
    let (root, config, files) = load_fixture("3304-wrappers");
    let findings = super::super::check_with_files(&root, &config, &files).unwrap();
    let mut observed: Vec<_> = findings
        .iter()
        .map(|finding| {
            let file = if finding.file.ends_with("ci.yml") {
                "ci.yml"
            } else if finding.file.ends_with("matrix.yml") {
                "matrix.yml"
            } else {
                panic!("unexpected workflow path: {finding:?}");
            };
            (file.to_string(), finding.line, spec_of(&finding.message))
        })
        .collect();
    observed.sort();
    assert_eq!(
        observed,
        [
            ("ci.yml".into(), 5, "1:2".into()),
            ("ci.yml".into(), 7, "1:2".into()),
            ("ci.yml".into(), 9, "15432:5432".into()),
            ("ci.yml".into(), 16, "9:5432/udp".into()),
            ("ci.yml".into(), 17, "[::1]:8:5432/tcp".into()),
            ("matrix.yml".into(), 5, "6001:5432".into()),
            ("matrix.yml".into(), 6, "6002:5432".into()),
            ("matrix.yml".into(), 7, "6003:5432".into()),
            ("matrix.yml".into(), 8, "6004:5432".into()),
            ("matrix.yml".into(), 9, "6005:5432".into()),
        ]
    );
}
