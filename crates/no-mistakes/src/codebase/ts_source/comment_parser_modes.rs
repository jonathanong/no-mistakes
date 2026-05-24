enum CodeCharAction {
    Comment(usize),
    Consumed,
}

struct CodeModeState<'a> {
    line_state: &'a mut LineCommentScanState,
    in_regex: &'a mut bool,
    in_unquoted_url: &'a mut bool,
    prev_significant: Option<char>,
}

fn consume_unquoted_url(ch: char, in_unquoted_url: &mut bool) -> bool {
    if !*in_unquoted_url {
        return false;
    }
    if unquoted_url_ends_before(ch) {
        *in_unquoted_url = false;
        false
    } else {
        true
    }
}

fn consume_block_comment(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    state: &mut LineCommentScanState,
) -> bool {
    if !state.in_block_comment {
        return false;
    }
    if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
        chars.next();
        state.in_block_comment = false;
    }
    true
}

fn consume_regex(
    ch: char,
    escaped: &mut bool,
    in_regex: &mut bool,
    in_regex_char_class: &mut bool,
    prev_significant: &mut Option<char>,
) -> bool {
    if !*in_regex {
        return false;
    }
    if *escaped {
        *escaped = false;
    } else if ch == '\\' {
        *escaped = true;
    } else if ch == '[' {
        *in_regex_char_class = true;
    } else if ch == ']' {
        *in_regex_char_class = false;
    } else if ch == '/' && !*in_regex_char_class {
        *in_regex = false;
        *prev_significant = Some('/');
    }
    true
}

fn consume_quote(
    ch: char,
    escaped: &mut bool,
    state: &mut LineCommentScanState,
    prev_significant: &mut Option<char>,
) -> bool {
    let Some(current) = state.quote else {
        return false;
    };
    if *escaped {
        *escaped = false;
    } else if ch == '\\' {
        *escaped = true;
    } else if ch == current {
        state.quote = None;
        *prev_significant = Some(current);
    }
    true
}

fn scan_code_char(
    line: &str,
    idx: usize,
    ch: char,
    next: Option<char>,
    mode: &mut CodeModeState<'_>,
) -> Option<CodeCharAction> {
    if matches!(ch, '\'' | '"' | '`') {
        mode.line_state.quote = Some(ch);
        return Some(CodeCharAction::Consumed);
    }
    if ch == '/' && next == Some('*') && block_comment_can_start(line, idx) {
        mode.line_state.in_block_comment = true;
        return Some(CodeCharAction::Consumed);
    }
    if ch == '/' && next == Some('/') {
        return Some(CodeCharAction::Comment(2));
    }
    if ch == '/' && regex_can_start_at(line, idx, mode.prev_significant) {
        *mode.in_regex = true;
        return Some(CodeCharAction::Consumed);
    }
    if ch == ':'
        && line[idx + ch.len_utf8()..].starts_with("//")
        && has_url_scheme_before_colon(line, idx)
    {
        *mode.in_unquoted_url = true;
        return Some(CodeCharAction::Consumed);
    }
    if ch == '#' && comment_prefix_can_start(line, idx) {
        return Some(CodeCharAction::Comment(1));
    }
    if ch == '-' && next == Some('-') && dash_comment_can_start(line, idx) {
        return Some(CodeCharAction::Comment(2));
    }
    None
}

fn block_comment_can_start_after_punctuation(ch: char) -> bool {
    matches!(
        ch,
        ';' | ','
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '&'
            | '|'
            | '^'
            | '<'
            | '>'
            | '!'
            | '?'
            | ':'
            | '('
            | ')'
            | '['
            | '{'
            | '.'
    )
}

fn regex_can_start_after_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '(' | '['
            | '{'
            | '='
            | ':'
            | ','
            | ';'
            | '!'
            | '?'
            | '>'
            | '<'
            | '+'
            | '-'
            | '*'
            | '%'
            | '&'
            | '|'
            | '^'
            | '~'
    )
}
