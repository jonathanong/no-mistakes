use crate::codebase::rules::github_actions_workflow_files::shell_words::{commands, Word};

pub(super) struct PollLoop {
    pub(super) line: usize,
    pub(super) keyword: String,
    pub(super) poll: String,
}

struct Frame {
    line: usize,
    keyword: String,
    sleeps: bool,
    poll: Option<String>,
}

const LEADING: &[&str] = &[
    "do", "then", "else", "elif", "if", "!", "{", "(", "time", "sudo", "command", "env", "nohup",
    "exec",
];

/// Loops (`while`, `until`, `for`) whose condition or body sleeps and runs a
/// configured poll command. An unbalanced script is not judged.
pub(super) fn poll_loops(script: &str, polls: &[Vec<String>], skip_for: bool) -> Vec<PollLoop> {
    let mut stack: Vec<Frame> = Vec::new();
    let mut found = Vec::new();
    for command in commands(script) {
        let start = command
            .iter()
            .position(|word| !matches!(word.text.as_str(), "do" | "then" | "else" | "elif" | "if"));
        let Some(start) = start else {
            continue;
        };
        let command = &command[start..];
        let first = &command[0];
        let head = first.text.as_str();
        if head == "done" {
            let Some(frame) = stack.pop() else {
                return Vec::new();
            };
            let reported = frame.sleeps && !(skip_for && frame.keyword == "for");
            if let (true, Some(poll)) = (reported, frame.poll.clone()) {
                found.push(PollLoop {
                    line: frame.line,
                    keyword: frame.keyword.clone(),
                    poll,
                });
            }
            continue;
        }
        let opens = matches!(head, "while" | "until" | "for");
        if opens {
            stack.push(Frame {
                line: first.line,
                keyword: head.to_string(),
                sleeps: false,
                poll: None,
            });
        }
        if head == "for" {
            continue;
        }
        let Some(frame) = stack.last_mut() else {
            continue;
        };
        inspect(&command[usize::from(opens)..], frame, polls);
    }
    if !stack.is_empty() {
        return Vec::new();
    }
    found.sort_by_key(|item| item.line);
    found
}

fn inspect(words: &[Word], frame: &mut Frame, polls: &[Vec<String>]) {
    let mut start = 0;
    while start < words.len() && is_leading(&words[start].text) {
        start += 1;
    }
    let words = &words[start..];
    if words
        .first()
        .is_some_and(|word| basename(&word.text) == "sleep")
    {
        frame.sleeps = true;
    }
    let flat = words
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let flat = flat.split_whitespace().collect::<Vec<_>>().join(" ");
    for poll in polls {
        let wanted = poll.join(" ");
        if at_command_position(&flat, &wanted) {
            frame.poll.get_or_insert(wanted);
        }
    }
}

fn is_leading(text: &str) -> bool {
    LEADING.contains(&text) || is_assignment(text)
}

fn is_assignment(text: &str) -> bool {
    let Some((name, value)) = text.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && !value.starts_with("$(")
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn at_command_position(flat: &str, wanted: &str) -> bool {
    flat.match_indices(wanted).any(|(index, _)| {
        let before = &flat[..index];
        let after = flat[index + wanted.len()..].chars().next();
        let ends = after.is_none_or(|ch| ch.is_whitespace() || matches!(ch, ')' | '`' | '"'));
        let starts = before.is_empty() || before.ends_with("$(") || before.ends_with('`');
        starts && ends
    })
}

fn basename(text: &str) -> &str {
    text.rsplit('/').next().unwrap_or(text)
}
