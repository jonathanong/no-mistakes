use super::has_unquoted_jsx_expression_brace;

#[test]
fn jsx_expression_braces_must_be_outside_quoted_attributes() {
    for opening in ["div value={value}>", "div value = {value}>"] {
        assert!(has_unquoted_jsx_expression_brace(opening));
    }
    for opening in [
        "div data-template=\"{name}\">",
        "div data-template='{name}'>",
        "div data-template=\"before > {name} after\">",
        "div>",
        "div",
    ] {
        assert!(!has_unquoted_jsx_expression_brace(opening));
    }
}
