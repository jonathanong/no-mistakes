use super::*;

#[test]
fn tracks_nested_expressions_without_counting_literal_or_comment_braces() {
    let mut scanner = MdxExpressionScanner::default();

    scanner.observe_line(b"{ nested: { value: '}' }, quoted: \"}\\\"\", template: `}`,");
    assert!(scanner.is_inside_expression());
    scanner.observe_line(b"/* } */ // }");
    assert!(scanner.is_inside_expression());
    scanner.observe_line(b"}");
    assert!(!scanner.is_inside_expression());
}

#[test]
fn keeps_multiline_literals_and_comments_inside_the_expression() {
    for lines in [
        &[b"{'first\\".as_slice(), b"line}'", b"}"][..],
        &[b"{\"first\\".as_slice(), b"line}\"", b"}"][..],
        &[b"{`first\\".as_slice(), b"line}`", b"}"][..],
        &[b"{/* first".as_slice(), b"} */", b"}"][..],
    ] {
        let mut scanner = MdxExpressionScanner::default();
        for line in &lines[..lines.len() - 1] {
            scanner.observe_line(line);
            assert!(scanner.is_inside_expression());
        }
        scanner.observe_line(lines[lines.len() - 1]);
        assert!(!scanner.is_inside_expression());
    }
}

#[test]
fn stops_gap_scanning_when_the_active_expression_closes() {
    let mut scanner = MdxExpressionScanner::default();

    scanner.observe_line(b"{`");
    scanner.observe_active_source(b"still inside the template\n`} then Markdown `literal { brace`");

    assert!(!scanner.is_inside_expression());
}
