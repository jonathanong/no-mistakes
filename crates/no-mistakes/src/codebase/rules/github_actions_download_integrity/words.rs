pub(super) struct Word {
    pub(super) text: String,
    pub(super) line: usize,
}

pub(super) struct Command {
    pub(super) dynamic: bool,
    pub(super) words: Vec<Word>,
}

pub(super) fn has_heredoc(script: &str) -> bool {
    let mut quote = None;
    let mut chars = script.chars().peekable();
    while let Some(ch) = chars.next() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '<' && chars.peek() == Some(&'<') {
            return true;
        }
    }
    false
}

pub(super) fn commands(script: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut current = Command {
        dynamic: false,
        words: Vec::new(),
    };
    let mut word = String::new();
    let mut word_line = 1usize;
    let mut quoting = None;
    let mut line = 1usize;
    let chars: Vec<char> = script.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if quoting.is_none() && ch == '\\' && chars.get(index + 1) == Some(&'\n') {
            index += 2;
            line += 1;
            continue;
        }
        if let Some(quote) = quoting {
            if ch == quote {
                quoting = None;
            } else {
                push_char(&mut word, &mut word_line, line, ch);
                if ch == '\n' {
                    line += 1;
                }
            }
            index += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            quoting = Some(ch);
            index += 1;
            continue;
        }
        if ch == '`' {
            current.dynamic = true;
        }
        if ch == '$' && matches!(chars.get(index + 1), Some(&'(' | &'{')) {
            current.dynamic = true;
            if word.is_empty() {
                word_line = line;
            }
            index = take_expansion(&chars, index, &mut word, &mut line);
            continue;
        }
        if ch == '\n' || ch == ';' || ch == '|' || (ch == '&' && chars.get(index + 1) == Some(&'&'))
        {
            flush_word(&mut word, &mut current, word_line);
            flush_command(&mut current, &mut commands);
            if ch == '&' || (ch == '|' && chars.get(index + 1) == Some(&'|')) {
                index += 1;
            }
            if ch == '\n' {
                line += 1;
            }
            index += 1;
            continue;
        }
        if ch.is_whitespace() || ch == '<' || ch == '>' {
            flush_word(&mut word, &mut current, word_line);
            index += 1;
            continue;
        }
        push_char(&mut word, &mut word_line, line, ch);
        index += 1;
    }
    flush_word(&mut word, &mut current, word_line);
    flush_command(&mut current, &mut commands);
    commands
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

fn flush_word(word: &mut String, current: &mut Command, word_line: usize) {
    if !word.is_empty() {
        current.words.push(Word {
            text: std::mem::take(word),
            line: word_line,
        });
    }
}

fn flush_command(current: &mut Command, commands: &mut Vec<Command>) {
    if current.dynamic || !current.words.is_empty() {
        commands.push(std::mem::replace(
            current,
            Command {
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

pub(super) fn command_name(words: &[Word]) -> Option<String> {
    command_at(words).map(|(name, _)| name)
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
