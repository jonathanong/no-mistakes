use super::super::comment::{parse_reason_text, ReasonStatus};
use super::lexers::{
    advance_utf8_char, find_eol, scan_block_comment, scan_dollar_quote_or_char, scan_js_quoted,
    scan_js_template, scan_sql_quoted,
};
use super::{CommentIndex, CommentSpan};

pub(super) fn eval_group(comments: &[&CommentSpan]) -> ReasonStatus {
    let mut best = ReasonStatus::Absent;
    let mut index = 0;
    while index < comments.len() {
        let comment = comments[index];
        if comment.is_line {
            let mut joined = comment.body.clone();
            let mut prev_line = comment.end_line;
            index += 1;
            while index < comments.len()
                && comments[index].is_line
                && comments[index].start_line == prev_line + 1
            {
                joined.push('\n');
                joined.push_str(&comments[index].body);
                prev_line = comments[index].end_line;
                index += 1;
            }
            best = best.merge(parse_reason_text(&joined));
        } else {
            best = best.merge(parse_reason_text(&comment.body));
            index += 1;
        }
        if best == ReasonStatus::Valid {
            return best;
        }
    }
    best
}

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
                let end = find_eol(bytes, start);
                col += end - i;
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
                let end = find_eol(bytes, start);
                col += end - i;
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
            b'\'' | b'"' => {
                index.mark_code(line, col);
                (i, line, col) = scan_js_quoted(bytes, i, line, col);
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
            }
            b'{' if !template_braces.is_empty() => {
                index.mark_code(line, col);
                *template_braces.last_mut().unwrap() += 1;
                col += 1;
                i += 1;
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
                } else {
                    *depth -= 1;
                    col += 1;
                    i += 1;
                }
            }
            _ => {
                index.mark_code(line, col);
                (i, col) = advance_utf8_char(source, i, col);
            }
        }
    }
    index
}
