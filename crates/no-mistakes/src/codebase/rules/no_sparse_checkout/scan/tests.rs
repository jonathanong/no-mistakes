use super::location::{checkout_key_line, yaml_key_value};

#[test]
fn checkout_key_line_finds_quoted_inputs_in_the_matching_checkout_step() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    with:\n      'sparse-checkout': src\n      \"sparse-checkout-cone-mode\": false\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 4);
    assert_eq!(checkout_key_line(source, "sparse-checkout-cone-mode", 0), 5);
}

#[test]
fn checkout_key_line_stops_when_the_with_mapping_ends() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    with:\n      fetch-depth: 0\n    run: echo done\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 2);
}

#[test]
fn checkout_key_line_stops_at_the_next_step() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    with:\n      fetch-depth: 0\n  - run: echo done\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 2);
}

#[test]
fn checkout_key_line_falls_back_after_an_unrelated_with_input() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    with:\n      fetch-depth: 0\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 2);
}

#[test]
fn checkout_key_line_falls_back_without_block_with_inputs() {
    let without_with = "steps:\n  - uses: actions/checkout@v4\n";
    assert_eq!(checkout_key_line(without_with, "sparse-checkout", 0), 2);

    let flow_with = "steps:\n  - uses: actions/checkout@v4\n    with: {}\n";
    assert_eq!(checkout_key_line(flow_with, "sparse-checkout", 0), 2);
}

#[test]
fn checkout_key_line_ignores_step_content_before_with() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    name: Checkout\n    with:\n      sparse-checkout: src\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 5);
}

#[test]
fn checkout_key_line_parses_multiline_flow_steps() {
    let source =
        "steps:\n  - { uses: actions/checkout@v4,\n      with: { sparse-checkout: src } }\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 2);
}

#[test]
fn checkout_key_line_handles_spaced_colons_and_indented_bare_steps() {
    let source = "steps :\n  -\n        uses : actions/checkout@v4\n        with :\n          'sparse-checkout' : src\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 5);
}

#[test]
fn checkout_key_line_parses_multiline_quoted_uses() {
    let source = "steps:\n  - uses: \"actions/checkout@v4\\\n      \"\n    with:\n      sparse-checkout: src\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 5);
}

#[test]
fn checkout_key_line_uses_only_direct_with_inputs() {
    let source = "steps:\n  - uses: actions/checkout@v4\n    with:\n      nested:\n        sparse-checkout: decoy\n      sparse-checkout: src\n";
    assert_eq!(checkout_key_line(source, "sparse-checkout", 0), 6);
}

#[test]
fn yaml_key_line_accepts_only_unquoted_or_quoted_exact_keys() {
    let has_key = |line| yaml_key_value(line, "sparse-checkout").is_some();
    assert!(has_key("sparse-checkout: src"));
    assert!(has_key("'sparse-checkout': src"));
    assert!(has_key("\"sparse-checkout\": src"));
    assert!(has_key("sparse-checkout : src"));
    assert!(has_key("'sparse-checkout' : src"));
    assert!(has_key("\"sparse-checkout\" : src",));
    assert!(!has_key("sparse-checkout-extra: src"));
}
