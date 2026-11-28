use std::ops::Range;

#[derive(PartialEq, Eq)]
pub(in crate::codebase::rules) struct Word {
    pub(in crate::codebase::rules) text: String,
    pub(in crate::codebase::rules) line: usize,
    /// An unquoted expansion may add fields and change following option slots.
    pub(in crate::codebase::rules) expansion_may_split: bool,
    pub(in crate::codebase::rules) quoted_expansion: Option<(usize, usize)>,
    /// The first simple reference outside quotes, as (byte offset, byte length).
    pub(in crate::codebase::rules) bare_expansion: Option<(usize, usize)>,
}

pub(in crate::codebase::rules) struct Command {
    // The preceding separator's character offset plus one, or zero at script start.
    pub(in crate::codebase::rules) anchor: usize,
    pub(in crate::codebase::rules) dynamic: bool,
    pub(in crate::codebase::rules) words: Vec<Word>,
}

mod scan;

pub(in crate::codebase::rules) struct HeredocBody {
    pub(in crate::codebase::rules) range: Range<usize>,
    pub(in crate::codebase::rules) quoted: bool,
}

pub(in crate::codebase::rules) struct Parse {
    pub(in crate::codebase::rules) commands: Vec<Command>,
    pub(in crate::codebase::rules) unterminated_heredoc: bool,
    pub(in crate::codebase::rules) unterminated_quote: bool,
    pub(in crate::codebase::rules) unterminated_substitution: bool,
    /// Nested bodies as `(line, char_start, body)` in the parent script.
    pub(in crate::codebase::rules) substitutions: Vec<(usize, usize, String)>,
    pub(super) skip_lines: Vec<usize>,
    // Character offsets in the unmodified shell source.
    pub(in crate::codebase::rules) heredoc_ranges: Vec<Range<usize>>,
    pub(in crate::codebase::rules) heredoc_bodies: Vec<HeredocBody>,
}

pub(in crate::codebase::rules) fn parse(script: &str) -> Parse {
    parse_ordered(script, false, false)
}

pub(in crate::codebase::rules) fn parse_ordered(script: &str, strict: bool, collect_skips: bool) -> Parse {
    scan::run(script, strict, collect_skips)
}

fn skip_arithmetic(chars: &[char], mut index: usize, line: &mut usize) -> usize {
    index += 2;
    let mut depth = 2i32;
    while index < chars.len() && depth > 0 {
        let ch = chars[index];
        if ch == '\n' {
            *line += 1;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        }
        index += 1;
    }
    index
}

fn take_expansion(
    chars: &[char],
    mut index: usize,
    word: &mut String,
    line: &mut usize,
) -> (usize, bool) {
    let open = chars[index + 1];
    let close = if open == '(' { ')' } else { '}' };
    word.push('$');
    word.push(open);
    index += 2;
    let mut depth = 1i32;
    while index < chars.len() && depth > 0 {
        let ch = chars[index];
        if ch == '\n' {
            *line += 1;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
        }
        word.push(ch);
        index += 1;
    }
    (index, depth == 0)
}

fn push_char(word: &mut String, word_line: &mut usize, line: usize, ch: char) {
    if word.is_empty() {
        *word_line = line;
    }
    word.push(ch);
}

fn flush_word(
    word: &mut String,
    current: &mut Command,
    word_line: usize,
    quoted: bool,
    expansion_may_split: &mut bool,
    quoted_expansion: &mut Option<(usize, usize)>,
    bare_expansion: &mut Option<(usize, usize)>,
) {
    let expansion_may_split = std::mem::take(expansion_may_split);
    let quoted_expansion = quoted_expansion.take();
    let bare_expansion = bare_expansion.take();
    if quoted || !word.is_empty() {
        current.words.push(Word {
            text: std::mem::take(word),
            line: word_line,
            expansion_may_split,
            quoted_expansion,
            bare_expansion,
        });
    }
}

fn flush_command(current: &mut Command, commands: &mut Vec<Command>) {
    if current.dynamic || !current.words.is_empty() {
        let anchor = current.anchor;
        commands.push(std::mem::replace(
            current,
            Command {
                anchor,
                dynamic: false,
                words: Vec::new(),
            },
        ));
    }
}

mod lead;

pub(in crate::codebase::rules) fn command_at(words: &[Word]) -> Option<(String, usize)> {
    lead::command_at(words)
}

pub(in crate::codebase::rules) fn is_assignment(text: &str) -> bool {
    let Some((name, _)) = text.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(in crate::codebase::rules) fn basename(text: &str) -> String {
    text.rsplit(['/', '\\']).next().unwrap_or(text).to_string()
}

pub(super) fn is_downloader(name: &str, helpers: &[String]) -> bool {
    name == "curl" || name == "wget" || helpers.iter().any(|helper| helper == name)
}
