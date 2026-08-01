use super::{has_unquoted_jsx_expression_brace, looks_like_mdx_flow_boundary};

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

#[test]
fn distinguishes_flow_mdx_jsx_from_inline_html() {
    for flow in [
        b"<Badge />".as_slice(),
        b"   <Badge />",
        b"<>",
        b"<div>",
        b"</SECTION>",
    ] {
        assert!(looks_like_mdx_flow_boundary(flow), "{flow:?}");
    }
    for inline in [
        b"paragraph <Badge />".as_slice(),
        b"<span>inline</span>",
        b"<a href='/'>inline</a>",
        b"<div.class>",
        b"<section-name>",
        b"    <Badge />",
        b"\t<Badge />",
    ] {
        assert!(!looks_like_mdx_flow_boundary(inline), "{inline:?}");
    }
}
