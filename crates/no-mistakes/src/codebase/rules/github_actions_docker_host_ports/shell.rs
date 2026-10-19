pub(super) struct Word {
    pub(super) text: String,
    pub(super) line: usize,
}

#[derive(Default)]
pub(super) struct Command {
    pub(super) words: Vec<Word>,
    pub(super) dynamic: bool,
}

pub(super) fn commands(script: &str) -> Vec<Command> {
    let chars: Vec<char> = script.chars().collect();
    let mut commands = Vec::new();
    let mut command = Command {
        words: Vec::new(),
        dynamic: false,
    };
    let mut word = String::new();
    let mut word_line = 1usize;
    let mut open = false;
    let mut line = 1usize;
    let mut quote: Option<char> = None;
    let mut subst = 0i32;
    let mut backtick = false;
    let mut index = 0;
    let flush_word =
        |word: &mut String, open: &mut bool, word_line: usize, command: &mut Command| {
            if *open {
                command.words.push(Word {
                    text: std::mem::take(word),
                    line: word_line,
                });
                *open = false;
            }
        };
    let flush_command = |command: &mut Command, commands: &mut Vec<Command>| {
        if command.dynamic || !command.words.is_empty() {
            commands.push(std::mem::take(command));
        }
    };
    while index < chars.len() {
        let ch = chars[index];
        if quote == Some('\'') {
            if ch == '\n' {
                line += 1;
            }
            if ch == '\'' {
                quote = None;
            } else {
                word.push(ch);
            }
            index += 1;
            continue;
        }
        if backtick {
            command.dynamic = true;
            if ch == '\n' {
                line += 1;
            }
            if ch == '`' {
                backtick = false;
            }
            index += 1;
            continue;
        }
        if subst > 0 {
            command.dynamic = true;
            if ch == '\n' {
                line += 1;
            }
            if ch == '(' {
                subst += 1;
            } else if ch == ')' {
                subst -= 1;
            }
            index += 1;
            continue;
        }
        if quote == Some('"') {
            if ch == '\\' && index + 1 < chars.len() {
                let next = chars[index + 1];
                if next == '\n' {
                    line += 1;
                } else {
                    word.push(next);
                }
                index += 2;
                continue;
            }
            if ch == '\n' {
                line += 1;
                word.push(ch);
                index += 1;
                continue;
            }
            if ch == '"' {
                quote = None;
                index += 1;
                continue;
            }
            if ch == '`' {
                command.dynamic = true;
                backtick = true;
                index += 1;
                continue;
            }
            if ch == '$' && chars.get(index + 1) == Some(&'(') {
                command.dynamic = true;
                subst = 1;
                index += 2;
                continue;
            }
            word.push(ch);
            index += 1;
            continue;
        }
        if ch == '\\' && chars.get(index + 1) == Some(&'\n') {
            line += 1;
            index += 2;
            continue;
        }
        if ch == '\\' && index + 1 < chars.len() {
            if !open {
                word_line = line;
                open = true;
            }
            word.push(chars[index + 1]);
            index += 2;
            continue;
        }
        if ch == '\'' || ch == '"' {
            if !open {
                word_line = line;
                open = true;
            }
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == '`' {
            command.dynamic = true;
            backtick = true;
            index += 1;
            continue;
        }
        if ch == '$' && chars.get(index + 1) == Some(&'(') {
            command.dynamic = true;
            subst = 1;
            index += 2;
            continue;
        }
        if ch == '#' && !open {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if ch == '\n' || ch == ';' || ch == '|' || ch == '&' || ch.is_whitespace() {
            flush_word(&mut word, &mut open, word_line, &mut command);
            if ch == '\n' {
                line += 1;
                flush_command(&mut command, &mut commands);
            } else if ch == ';' || ch == '|' || ch == '&' {
                if (ch == '|' || ch == '&') && chars.get(index + 1) == Some(&ch) {
                    index += 1;
                }
                flush_command(&mut command, &mut commands);
            }
            index += 1;
            continue;
        }
        if !open {
            word_line = line;
            open = true;
        }
        word.push(ch);
        index += 1;
    }
    if quote.is_some() || backtick || subst > 0 {
        command.dynamic = true;
    }
    flush_word(&mut word, &mut open, word_line, &mut command);
    flush_command(&mut command, &mut commands);
    commands
}
