use super::*;

#[test]
fn dense_shape_divergence_keeps_legacy_unknowns_and_independent_queries_exact() {
    let (path, mut files) = prepared(&["shape-divergence.mts"]);
    let source = std::fs::read_to_string(&path).unwrap();
    let mut calls = project_executed(&mut files, |_: &str, _: &Path| None);
    assert_eq!(
        calls.len(),
        5,
        "the saved file has five executed query sites"
    );
    let mut at = |expression: &str| {
        let start = source
            .find(expression)
            .expect("saved exact query expression") as u32;
        calls
            .remove(&(path.clone(), (start, start + expression.len() as u32)))
            .expect("the public projection retains every saved call site")
    };
    // Frozen main104466 CLI with saved shape.yml reports all three as
    // unanalyzable, including sparse reads after the dense sentinel appears.
    // Preserve that conservative boundary rather than inventing new precision.
    for expression in [
        "query(values[0])",
        "query(values[1001])",
        "query(values[1000])",
    ] {
        assert!(
            at(expression).is_err(),
            "legacy unanalyzable site: {expression}"
        );
    }
    for text in ["SELECT sibling baseline", "SELECT root baseline"] {
        assert_eq!(
            at(&format!("query(\"{text}\")")),
            Ok(vec![super::super::ProjectedSql {
                text: text.to_string(),
                complete: true,
                only_value_placeholders: false,
            }]),
            "independent literal execution remains exact"
        );
    }
    assert!(calls.is_empty());
}
