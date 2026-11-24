use super::*;

const EDGES: &str = include_str!("../../../fixtures/shell-heredoc/edges.sh");

#[test]
fn shift_and_here_string_operators_do_not_open_documents() {
    let lines: Vec<_> = EDGES.lines().collect();
    let (_, here_string) = lines[0].split_once(' ').unwrap();
    assert_eq!(
        operator(here_string.as_bytes(), false),
        Some(Operator::HereString)
    );
    assert!(line_opener(lines[0]).is_none());
    assert!(line_opener(lines[1]).is_none());
    let shift = &lines[2][lines[2].find("<<").unwrap()..];
    assert_eq!(operator(shift.as_bytes(), true), None);
    assert_eq!(
        operator(shift.as_bytes(), false),
        Some(Operator::Document { strip_tabs: false })
    );
}

#[test]
fn quoted_and_tab_stripped_delimiters_close_in_order() {
    let mut lines = EDGES.splitn(5, '\n');
    let opener = lines.nth(3).unwrap();
    let (first, quoted) = line_opener(opener).unwrap();
    assert_eq!(first.delimiter, "FIRST");
    assert!(quoted && first.strip_tabs);
    let second: Vec<_> = opener.rsplit_once("<<").unwrap().1.chars().collect();
    assert_eq!(
        char_delimiter(&second, DelimiterStyle::LeadingQuote),
        ("SECOND".into(), second.len())
    );
    assert_eq!(
        char_delimiter(&second, DelimiterStyle::StripQuotes),
        ("SECOND".into(), second.len())
    );

    let source = lines.next().unwrap();
    let chars: Vec<_> = source.chars().collect();
    let body = skip_body(
        &chars,
        0,
        first.delimiter.chars(),
        first.strip_tabs,
        CarriageReturn::Keep,
    );
    assert_eq!(body.newlines, 2);
    assert!(body.terminated);
    let body = skip_body(
        &chars,
        body.end,
        "SECOND".chars(),
        false,
        CarriageReturn::Keep,
    );
    assert_eq!(body.newlines, 2);
    assert!(body.terminated);
    assert!(chars[body.end..].starts_with(&['c', 'a', 't']));
}

#[test]
fn closing_line_policies_preserve_crlf_and_end_of_input() {
    let (_, body) = EDGES.split_once("payload one").unwrap();
    let (first, _) = body.split_once("payload two").unwrap();
    let crlf = first.replace('\n', "\r\n");
    let input = crlf.as_bytes();
    assert!(!skip_body(input, 0, "FIRST".bytes(), true, CarriageReturn::Keep).terminated);
    assert!(
        skip_body(
            input,
            0,
            "FIRST".bytes(),
            true,
            CarriageReturn::BeforeNewline
        )
        .terminated
    );

    let eof = crlf.strip_suffix('\n').unwrap().as_bytes();
    let body = skip_body(eof, 0, "FIRST".bytes(), true, CarriageReturn::StripTrailing);
    assert_eq!(
        body,
        Body {
            end: eof.len(),
            newlines: 1,
            terminated: true
        }
    );
    assert!(!skip_body(eof, 0, "FIRST".bytes(), true, CarriageReturn::BeforeNewline).terminated);
}

#[test]
fn bare_opener_and_unterminated_body_keep_the_callers_recovery() {
    let (_, tail) = EDGES.split_once("cat <<\n").unwrap();
    assert!(line_opener("cat <<").is_none());
    let (opener, source) = tail.split_once('\n').unwrap();
    let (doc, quoted) = line_opener(opener).unwrap();
    assert!(quoted);
    let body = skip_body(
        source.as_bytes(),
        0,
        doc.delimiter.bytes(),
        doc.strip_tabs,
        CarriageReturn::Keep,
    );
    assert_eq!(
        body,
        Body {
            end: source.len(),
            newlines: 1,
            terminated: false
        }
    );
    let lines: Vec<_> = source.lines().collect();
    assert_eq!(trimmed_body_lines(&lines, &doc), (lines.as_slice(), 1));
}
