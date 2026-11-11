use super::read;

#[test]
fn argument_text_truncation_stops_before_multibyte_characters() {
    let read = read("utf8-arguments.mjs");
    let mut texts = read
        .provenance()
        .arg_text
        .values()
        .map(|arguments| arguments[0].text.as_str())
        .collect::<Vec<_>>();
    texts.sort();
    assert_eq!(
        texts,
        [
            format!("\"{}", "a".repeat(78)),
            format!("\"{}", "b".repeat(77))
        ]
    );
    assert!(texts.iter().all(|text| text.len() <= 80));
}

#[test]
fn argument_lines_share_one_index_across_calls_and_properties() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let read =
        crate::diagnostics::with_observer(Some(observer.clone()), || read("argument-lines.mjs"));
    assert_eq!(observer.snapshot().work["facts.argument_line_indexes"], 1);
    let mut calls: Vec<_> = read.provenance().arg_text.iter().collect();
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
