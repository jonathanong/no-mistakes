use super::super::options::CompiledOptions;

const WRAPPERS_AND_SHELLS: &[&str] = &[
    "bash", "sh", "zsh", "dash", "ksh", "fish", "pwsh", "env", "sudo", "xargs", "nohup", "time",
    "exec", "command",
];

pub(super) fn abstains_on_literal_script_path(text: &str, compiled: &CompiledOptions) -> bool {
    let Some(segment) = literal_final_path_segment(text) else {
        return false;
    };
    !WRAPPERS_AND_SHELLS.contains(&segment)
        && !compiled
            .commands
            .iter()
            .any(|policy| policy.command[0] == segment)
}

fn literal_final_path_segment(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut closes = Vec::new();
    let mut in_backticks = false;
    let mut last_sep = None;
    while i < bytes.len() {
        match bytes[i] {
            b'`' => {
                in_backticks = !in_backticks;
                i += 1;
            }
            b'$' if !in_backticks && matches!(bytes.get(i + 1), Some(b'{' | b'(')) => {
                let open = bytes[i + 1];
                let close = if open == b'{' { b'}' } else { b')' };
                closes.push(close);
                i += 2;
                if open == b'{' && bytes.get(i) == Some(&b'{') {
                    closes.push(b'}');
                    i += 1;
                }
            }
            ch if !in_backticks && closes.last() == Some(&ch) => {
                closes.pop();
                i += 1;
            }
            b'/' | b'\\' if !in_backticks && closes.is_empty() => {
                last_sep = Some(i);
                i += 1;
            }
            _ => i += 1,
        }
    }
    if in_backticks || !closes.is_empty() {
        return None;
    }
    let tail = &text[last_sep? + 1..];
    (!tail.is_empty() && !tail.contains(['$', '`', '*', '?', '[', ']', '{', '}', '(', ')']))
        .then_some(tail)
}
