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
