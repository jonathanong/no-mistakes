use super::extract_with_arguments;

#[test]
fn argument_lines_share_one_index_across_calls_and_properties() {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("argument-lines.mjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::ast::with_program(&path, &source, |program, _| {
            extract_with_arguments(program, true)
        })
        .unwrap()
    });
    assert_eq!(observer.snapshot().work["facts.argument_line_indexes"], 1);
    let provenance = facts.path_provenance.as_deref().unwrap();
    let mut calls: Vec<_> = provenance.arg_text.iter().collect();
    calls.sort_by_key(|(offset, _)| **offset);
    assert_eq!(calls.len(), 34);
    for (index, (_, arguments)) in calls.iter().take(32).enumerate() {
        assert_eq!(arguments[0].line, index as u32 + 3);
        assert_eq!(arguments[0].text, (index + 1).to_string());
    }
    let arguments = calls[32].1;
    let root = arguments
        .iter()
        .find(|arg| arg.property.as_deref() == Some("root"))
        .unwrap();
    assert_eq!((root.line, root.text.as_str()), (38, "root"));
    let label = arguments
        .iter()
        .find(|arg| arg.property.as_deref() == Some("label"))
        .unwrap();
    assert_eq!((label.line, label.text.as_str()), (36, "'é🙂'"));
    assert_eq!(
        (calls[33].1[0].line, calls[33].1[0].text.as_str()),
        (41, "label")
    );
}
