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
    scanner.observe_source(b"still inside the template\n`} then Markdown `literal { brace`");

    assert!(!scanner.is_inside_expression());
}

#[test]
fn ignores_braces_inside_regex_literals() {
    for expression in [
        b"{/[{]/.test(value)}".as_slice(),
        b"{/\\}/g.test(value)}",
        b"{return /[}]/i}",
        b"{condition ? /{/ : /}/}",
    ] {
        let mut scanner = MdxExpressionScanner::default();
        scanner.observe_line(expression);
        assert!(!scanner.is_inside_expression(), "{expression:?}");
    }

    // JavaScript regex literals cannot cross a raw newline; recover so one
    // malformed line cannot mask every later Markdown fence.
    let mut scanner = MdxExpressionScanner::default();
    scanner.observe_line(b"{/[{]");
    assert!(scanner.is_inside_expression());
    scanner.observe_line(b"}");
    assert!(!scanner.is_inside_expression());
}

#[test]
fn distinguishes_division_from_regex_literals() {
    for expression in [
        b"{value / divisor + ({ nested: 1 }).nested}".as_slice(),
        b"{value++ / divisor}",
        b"{value-- / divisor}",
    ] {
        let mut scanner = MdxExpressionScanner::default();

        scanner.observe_line(expression);

        assert!(!scanner.is_inside_expression(), "{expression:?}");
    }
}

#[test]
fn discovers_top_level_expressions_and_esm_regions() {
    for lines in [
        &[b"{".as_slice(), b"  `~~~mermaid", b"invalid", b"~~~`", b"}"][..],
        &[
            b"\xef\xbb\xbfexport const example =".as_slice(),
            b"`",
            b"~~~mermaid",
            b"invalid",
            b"~~~`;",
        ][..],
        &[
            b"import {".as_slice(),
            b"  example",
            b"} from './example.js'",
        ][..],
        &[b"export const values = (".as_slice(), b"  [example]", b")"][..],
    ] {
        let mut scanner = MdxExpressionScanner::default();
        for line in &lines[..lines.len() - 1] {
            scanner.observe_source(line);
            assert!(scanner.is_inside_expression(), "{line:?}");
        }
        scanner.observe_source(lines[lines.len() - 1]);
        assert!(!scanner.is_inside_expression(), "{lines:?}");
    }

    let mut scanner = MdxExpressionScanner::default();
    scanner.observe_source(b"important prose\nexported value\n");
    assert!(!scanner.is_inside_expression());
}

#[test]
fn masks_multiline_jsx_quoted_attributes() {
    let mut scanner = MdxExpressionScanner::default();

    scanner.observe_line(b"<DiagramCard description=\"");
    assert!(scanner.is_masking_markdown());
    scanner.observe_line(b"```mermaid");
    assert!(scanner.is_masking_markdown());
    scanner.observe_line(b"\" />");
    assert!(!scanner.is_masking_markdown());
    assert!(!scanner.jsx_opening);
}
