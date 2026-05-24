fn leading_comment_text(line: &str) -> Option<&str> {
    line.strip_prefix("//")
        .or_else(|| line.strip_prefix('#'))
        .or_else(|| {
            let rest = line.strip_prefix("--")?;
            rest.chars().next().is_some_and(char::is_whitespace).then_some(rest)
        })
        .map(str::trim)
}

#[derive(Default)]
struct LineCommentScanState {
    in_block_comment: bool,
    quote: Option<char>,
}

fn line_comment_start(
    line: &str,
    state: &mut LineCommentScanState,
) -> Option<(usize, usize)> {
    let mut escaped = false;
    let mut in_regex = false;
    let mut in_regex_char_class = false;
    let mut in_unquoted_url = false;
    let mut prev_significant = None;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if consume_unquoted_url(ch, &mut in_unquoted_url)
            || consume_block_comment(ch, &mut chars, state)
            || consume_regex(
                ch,
                &mut escaped,
                &mut in_regex,
                &mut in_regex_char_class,
                &mut prev_significant,
            )
            || consume_quote(ch, &mut escaped, state, &mut prev_significant)
        {
            continue;
        }
        let mut mode = CodeModeState {
            line_state: state,
            in_regex: &mut in_regex,
            in_unquoted_url: &mut in_unquoted_url,
            prev_significant,
        };
        match scan_code_char(
            line,
            idx,
            ch,
            chars.peek().map(|(_, next)| *next),
            &mut mode,
        ) {
            Some(CodeCharAction::Comment(prefix_len)) => return Some((idx, prefix_len)),
            Some(CodeCharAction::Consumed) => continue,
            None => {}
        }
        if !ch.is_whitespace() {
            prev_significant = Some(ch);
        }
    }
    None
}

fn unquoted_url_ends_before(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ';' | '&' | '|' | '(' | ')')
}

fn block_comment_can_start(line: &str, idx: usize) -> bool {
    line[idx + 2..].contains("*/")
        || line[..idx].trim().is_empty()
        || code_before_block_comment_can_start(line, idx)
}

fn code_before_block_comment_can_start(line: &str, idx: usize) -> bool {
    let before = line[..idx].trim_end();
    before.contains('=')
        || before
            .chars()
            .next_back()
            .is_some_and(block_comment_can_start_after_punctuation)
        || line[..idx].chars().next_back().is_some_and(is_word_char)
        || previous_word(line, idx).is_some_and(|word| {
            matches!(
                word,
                "const" | "let" | "var" | "return" | "if" | "else" | "for" | "while" | "do"
            )
        })
}

fn comment_prefix_can_start(line: &str, idx: usize) -> bool {
    line[..idx]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, ';' | '&' | '|' | '(' | ')'))
}

fn dash_comment_can_start(line: &str, idx: usize) -> bool {
    line[idx + 2..]
        .chars()
        .next()
        .is_some_and(char::is_whitespace)
        && (comment_prefix_can_start(line, idx)
            || line[..idx]
                .chars()
                .next_back()
                .is_some_and(|ch| ch == '_' || ch.is_ascii_alphanumeric()))
}

fn has_url_scheme_before_colon(line: &str, idx: usize) -> bool {
    let scheme_start = line[..idx]
        .char_indices()
        .rev()
        .find(|(_, ch)| !matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '+' | '.' | '-'))
        .map_or(0, |(idx, ch)| idx + ch.len_utf8());
    let scheme = &line[scheme_start..idx];
    scheme
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
}

fn regex_can_start_at(line: &str, idx: usize, prev: Option<char>) -> bool {
    prev.is_none_or(regex_can_start_after_punctuation)
        || prev == Some(')') && paren_closes_control_flow_condition(line, idx)
        || previous_word(line, idx).is_some_and(regex_can_start_after_word)
}

fn paren_closes_control_flow_condition(line: &str, idx: usize) -> bool {
    let before = line[..idx].trim_end();
    let mut depth = 0;
    for (open_idx, ch) in before.char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => {
                depth -= 1;
                if depth == 0 {
                    return paren_open_follows_control_flow(line, open_idx);
                }
            }
            _ => {}
        }
    }
    false
}

fn paren_open_follows_control_flow(line: &str, open_idx: usize) -> bool {
    matches!(
        previous_word(line, open_idx),
        Some("if" | "while" | "for" | "with" | "switch")
    )
}

fn regex_can_start_after_word(word: &str) -> bool {
    matches!(
        word,
        "return" | "throw" | "case" | "delete" | "typeof" | "void" | "yield" | "await" | "in"
            | "of" | "new"
    )
}

fn previous_word(line: &str, idx: usize) -> Option<&str> {
    let before = line[..idx].trim_end();
    let end = before
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_word_char(*ch))
        .map_or(0, |(idx, ch)| idx + ch.len_utf8());
    let word = &before[end..];
    (!word.is_empty()).then_some(word)
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}
