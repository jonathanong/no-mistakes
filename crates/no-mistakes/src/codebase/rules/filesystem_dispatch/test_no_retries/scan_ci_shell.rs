use super::scan_ci_lexer::{split_shell_segments, Sep};

const RUNNER_PREFIXES: &[&str] = &[
    "pnpm", "npx", "npm", "yarn", "bunx", "bun", "uv", "bundle", "exec", "dlx", "run", "x",
];

pub(super) struct ShellRerunHit {
    pub(super) line: usize,
    pub(super) token: String,
    pub(super) command: String,
    pub(super) shape: &'static str,
}

pub(super) fn find_shell_reruns(script: &str, test_commands: &[Vec<String>]) -> Vec<ShellRerunHit> {
    let mut loops: Vec<bool> = Vec::new();
    let mut prev_matched = false;
    let mut hits = Vec::new();
    for seg in split_shell_segments(script) {
        let slice = strip_control_keywords(&seg.words, &mut loops);
        let matched = match_test_command(slice, test_commands);
        if let Some(cmd) = &matched {
            let shape = if loops.iter().any(|&repeating| repeating) {
                Some("a shell loop")
            } else if seg.preceded_by == Sep::Or && prev_matched {
                Some("an `||` fallback rerun")
            } else {
                None
            };
            if let Some(shape) = shape {
                hits.push(ShellRerunHit {
                    line: seg.line,
                    token: slice.first().cloned().unwrap_or_default(),
                    command: cmd.clone(),
                    shape,
                });
            }
        }
        prev_matched = matched.is_some();
    }
    hits
}

fn strip_control_keywords<'a>(mut slice: &'a [String], loops: &mut Vec<bool>) -> &'a [String] {
    while let Some(first) = slice.first().map(String::as_str) {
        match first {
            "for" => {
                loops.push(for_header_can_repeat(&slice[1..]));
                return &[];
            }
            "until" | "while" => loops.push(true),
            "do" | "then" | "else" | "elif" | "!" => {}
            "done" => {
                loops.pop();
            }
            _ => break,
        }
        slice = &slice[1..];
    }
    slice
}

fn for_header_can_repeat(rest: &[String]) -> bool {
    let Some(in_idx) = rest.iter().position(|w| w == "in") else {
        return true;
    };
    let items = &rest[in_idx + 1..];
    if items.len() != 1 {
        return items.len() > 1;
    }
    items[0].contains(['*', '?', '[', '$', '`', '{'])
}

fn match_test_command(words: &[String], configured: &[Vec<String>]) -> Option<String> {
    let clean: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .skip_while(|w| w.contains('='))
        .collect();
    if let Some(cmd) = configured.iter().find(|c| starts_with_tokens(&clean, c)) {
        return Some(cmd.join(" "));
    }
    let stripped: Vec<&str> = clean
        .iter()
        .copied()
        .skip_while(|w| RUNNER_PREFIXES.contains(w))
        .collect();
    configured
        .iter()
        .find(|c| starts_with_tokens(&stripped, c))
        .map(|c| c.join(" "))
}

fn starts_with_tokens(words: &[&str], needle: &[String]) -> bool {
    !needle.is_empty()
        && words.len() >= needle.len()
        && words
            .iter()
            .zip(needle)
            .all(|(word, target)| *word == target.as_str())
}
