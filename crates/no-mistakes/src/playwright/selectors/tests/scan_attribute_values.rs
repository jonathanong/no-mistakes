use crate::playwright::selectors::scan_selector_attribute_values;

fn attrs() -> Vec<String> {
    vec!["data-pw".to_string(), "data-testid".to_string()]
}

#[test]
fn captures_rename_pair_in_removed_line() {
    let lines = vec!["  <form data-pw=\"search-bar\">".to_string()];
    let got = scan_selector_attribute_values(&attrs(), &lines);
    assert_eq!(got, vec![("data-pw".to_string(), "search-bar".to_string())]);
}

#[test]
fn captures_multiple_attribute_values_in_one_line() {
    let lines = vec!["<button data-pw=\"a\" data-testid=\"b\">".to_string()];
    let got = scan_selector_attribute_values(&attrs(), &lines);
    assert!(got.contains(&("data-pw".to_string(), "a".to_string())));
    assert!(got.contains(&("data-testid".to_string(), "b".to_string())));
}

#[test]
fn handles_single_quotes() {
    let lines = vec!["<x data-pw='val'/>".to_string()];
    let got = scan_selector_attribute_values(&attrs(), &lines);
    assert_eq!(got, vec![("data-pw".to_string(), "val".to_string())]);
}

#[test]
fn skips_dynamic_template_values() {
    let lines = vec!["<x data-pw={`x-${id}`}>".to_string()];
    let got = scan_selector_attribute_values(&attrs(), &lines);
    assert!(
        got.is_empty(),
        "dynamic values must be skipped, got: {:?}",
        got
    );
}

#[test]
fn skips_unconfigured_attributes() {
    let attrs = vec!["data-pw".to_string()];
    let lines = vec!["<x data-other=\"x\" data-pw=\"y\"/>".to_string()];
    let got = scan_selector_attribute_values(&attrs, &lines);
    assert_eq!(got, vec![("data-pw".to_string(), "y".to_string())]);
}

#[test]
fn empty_inputs_return_empty() {
    let empty_attrs: Vec<String> = Vec::new();
    let lines = vec!["data-pw=\"x\"".to_string()];
    assert!(scan_selector_attribute_values(&empty_attrs, &lines).is_empty());
    let lines: Vec<String> = Vec::new();
    assert!(scan_selector_attribute_values(&attrs(), &lines).is_empty());
}
