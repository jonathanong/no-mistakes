use std::ops::Range;

#[derive(PartialEq, Eq)]
pub(super) struct Word {
    pub(super) text: String,
    pub(super) line: usize,
    /// An unquoted expansion may add fields and change following option slots.
    pub(super) expansion_may_split: bool,
    pub(super) quoted_expansion: Option<(usize, usize)>,
    /// The first simple reference outside quotes, as (byte offset, byte length).
    pub(super) bare_expansion: Option<(usize, usize)>,
}

pub(super) struct Command {
    // The preceding separator's character offset plus one, or zero at script start.
    pub(super) anchor: usize,
    pub(super) dynamic: bool,
    pub(super) words: Vec<Word>,
}

mod heredoc;
mod scan;

pub(super) struct Parse {
    pub(super) commands: Vec<Command>,
    pub(super) unterminated_heredoc: bool,
    pub(super) skip_lines: Vec<usize>,
    // Character offsets in the unmodified shell source.
    pub(super) heredoc_ranges: Vec<Range<usize>>,
}

pub(super) fn parse(script: &str) -> Parse {
    parse_ordered(script, false, false)
}

pub(super) fn parse_ordered(script: &str, strict: bool, collect_skips: bool) -> Parse {
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

fn take_expansion(chars: &[char], mut index: usize, word: &mut String, line: &mut usize) -> usize {
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
    index
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

pub(super) fn command_at(words: &[Word]) -> Option<(String, usize)> {
    lead::command_at(words)
}

pub(super) fn is_assignment(text: &str) -> bool {
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

pub(super) fn basename(text: &str) -> String {
    text.rsplit(['/', '\\']).next().unwrap_or(text).to_string()
}

pub(super) fn is_downloader(name: &str, helpers: &[String]) -> bool {
    name == "curl" || name == "wget" || helpers.iter().any(|helper| helper == name)
}
