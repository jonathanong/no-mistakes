use super::*;
use std::collections::BTreeMap;

#[test]
fn preserved_projection_cases_keep_selected_occurrences_and_conservative_controls() {
    assert_preserved_fixture("superseded-2065-projections-fixture", 10);
}

#[test]
fn preserved_custom_import_cases_keep_hoisted_identity_and_lexical_shadows() {
    assert_preserved_fixture("superseded-2065-import-order-fixture", 3);
}

fn assert_preserved_fixture(fixture: &str, selected: usize) {
    let root = root().with_file_name(fixture);
    let expected: BTreeMap<String, Vec<usize>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected-lines.json")).unwrap())
            .unwrap();
    assert_eq!(expected.values().map(Vec::len).sum::<usize>(), selected);
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    let mut actual = BTreeMap::<String, Vec<usize>>::new();
    for finding in findings {
        actual.entry(finding.file).or_default().push(finding.line);
    }
    let positive = expected
        .iter()
        .filter(|(_, lines)| !lines.is_empty())
        .map(|(file, lines)| (file.clone(), lines.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, positive);
    for (file, lines) in expected {
        let report =
            crate::effects_query::run(&root, None, None, "postgres", Path::new(&file), &[], None)
                .unwrap();
        assert_eq!(
            report
                .call_sites
                .iter()
                .map(|site| (site.file.as_str(), site.line))
                .collect::<Vec<_>>(),
            lines
                .iter()
                .map(|line| (file.as_str(), *line))
                .collect::<Vec<_>>(),
            "{file}: {report:#?}"
        );
    }
}

#[test]
fn preserved_plain_non_nullish_report_uses_one_prepared_graph_and_parse() {
    let root = root().with_file_name("superseded-2065-plain-fixture");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let report = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::effects_query::run(
            &root,
            None,
            None,
            "database",
            Path::new("src/defaults.mts"),
            &[],
            None,
        )
    })
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(report).unwrap(), expected);
    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(work["parse.files"], 1, "{work:#?}");
}
