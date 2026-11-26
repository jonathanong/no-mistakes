use super::js_lexers::{regex_can_follow_word, scan_js_quoted, scan_js_regex, scan_js_template};
use super::lexers::{
    advance_utf8_char, find_eol, is_ident, scan_block_comment, scan_dollar_quote_or_char,
    scan_sql_quoted,
};
use super::CommentIndex;

pub(super) fn for_sql(source: &str) -> CommentIndex {
    let mut index = CommentIndex::empty(source);
    let bytes = source.as_bytes();
    let (mut i, mut line, mut col) = (0usize, 1usize, 1usize);
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                line += 1;
                col = 1;
                i += 1;
            }
            b' ' | b'\t' | b'\r' => {
                col += 1;
                i += 1;
            }
            b'-' if bytes.get(i + 1) == Some(&b'-') => {
                let (sl, sc) = (line, col);
                let start = i + 2;
                let (end, chars) = find_eol(bytes, start);
                col += 2 + chars;
                index.push_comment(sl, sc, line, col, &source[start..end], true);
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let (sl, sc) = (line, col);
                let start = i + 2;
                let (body_end, next_i, nl, nc) = scan_block_comment(bytes, i, line, col, true);
                index.push_comment(sl, sc, nl, nc, &source[start..body_end], false);
                (i, line, col) = (next_i, nl, nc);
            }
            b'\'' | b'"' => {
                index.mark_code(line, col);
                (i, line, col) = scan_sql_quoted(bytes, i, line, col);
                index.mark_code(line, col.saturating_sub(1).max(1));
            }
            b'$' => {
                index.mark_code(line, col);
                (i, line, col) = scan_dollar_quote_or_char(bytes, i, line, col);
                index.mark_code(line, col.saturating_sub(1).max(1));
            }
            b';' => {
                index.mark_code(line, col);
                index.semicolons.push((line, col));
                col += 1;
                i += 1;
            }
            _ => {
                index.mark_code(line, col);
                (i, col) = advance_utf8_char(source, i, col);
            }
        }
    }
    index
}

pub(super) fn for_ts(source: &str) -> CommentIndex {
    let mut index = CommentIndex::empty(source);
    let bytes = source.as_bytes();
    let (mut i, mut line, mut col) = (0usize, 1usize, 1usize);
    let mut template_braces: Vec<usize> = Vec::new();
    let mut can_start_regex = true;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                line += 1;
                col = 1;
                i += 1;
            }
            b' ' | b'\t' | b'\r' => {
                col += 1;
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let (sl, sc) = (line, col);
                let start = i + 2;
                let (end, chars) = find_eol(bytes, start);
                col += 2 + chars;
                index.push_comment(sl, sc, line, col, &source[start..end], true);
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let (sl, sc) = (line, col);
                let start = i + 2;
                let (body_end, next_i, nl, nc) = scan_block_comment(bytes, i, line, col, false);
                index.push_comment(sl, sc, nl, nc, &source[start..body_end], false);
                (i, line, col) = (next_i, nl, nc);
            }
            b'/' if can_start_regex => {
                index.mark_code(line, col);
                (i, line, col) = scan_js_regex(bytes, i, line, col);
                index.mark_code(line, col.saturating_sub(1).max(1));
                can_start_regex = false;
            }
            b'\'' | b'"' => {
                index.mark_code(line, col);
                (i, line, col) = scan_js_quoted(bytes, i, line, col);
                index.mark_code(line, col.saturating_sub(1).max(1));
                can_start_regex = false;
            }
            b'`' => {
                index.mark_code(line, col);
                (i, line, col) = scan_js_template(
                    bytes,
                    i + 1,
                    line,
                    col + 1,
                    &mut index,
                    &mut template_braces,
                );
                can_start_regex = !template_braces.is_empty();
            }
            b';' => {
                index.mark_code(line, col);
                index.semicolons.push((line, col));
                col += 1;
                i += 1;
                can_start_regex = true;
            }
            b'{' if !template_braces.is_empty() => {
                index.mark_code(line, col);
                *template_braces.last_mut().unwrap() += 1;
                col += 1;
                i += 1;
                can_start_regex = true;
            }
            b'}' if !template_braces.is_empty() => {
                index.mark_code(line, col);
                let depth = template_braces.last_mut().unwrap();
                if *depth == 0 {
                    template_braces.pop();
                    (i, line, col) = scan_js_template(
                        bytes,
                        i + 1,
                        line,
                        col + 1,
                        &mut index,
                        &mut template_braces,
                    );
                    can_start_regex = !template_braces.is_empty();
                } else {
                    *depth -= 1;
                    col += 1;
                    i += 1;
                    can_start_regex = false;
                }
            }
            b if is_ident(b) || b == b'$' => {
                let word_start = i;
                index.mark_code(line, col);
                (i, col) = advance_utf8_char(source, i, col);
                while i < bytes.len() && (is_ident(bytes[i]) || bytes[i] == b'$') {
                    index.mark_code(line, col);
                    (i, col) = advance_utf8_char(source, i, col);
                }
                can_start_regex = regex_can_follow_word(&source[word_start..i]);
            }
            b => {
                index.mark_code(line, col);
                can_start_regex = !matches!(b, b')' | b']' | b'}');
                (i, col) = advance_utf8_char(source, i, col);
            }
        }
    }
    index
}
