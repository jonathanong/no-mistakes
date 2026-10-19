use super::super::flags::is_assign;
use super::super::shell::Word;

const LEADING: &[&str] = &[
    "if", "then", "elif", "else", "do", "while", "until", "!", "{", "(", "time", "exec", "command",
    "nohup",
];

pub(super) fn command_text(text: &str) -> &str {
    text.trim_start_matches('(').trim_end_matches(')')
}

pub(super) fn skip_prefix(words: &[Word]) -> usize {
    let mut index = 0;
    while let Some(word) = words.get(index) {
        let text = command_text(&word.text);
        if text.is_empty() || is_assign(&word.text) || is_assign(text) || LEADING.contains(&text) {
            index += 1;
            continue;
        }
        if text == "sudo" {
            index = skip_sudo(words, index + 1);
            continue;
        }
        break;
    }
    index
}

fn skip_sudo(words: &[Word], mut index: usize) -> usize {
    while let Some(flag) = words.get(index) {
        match flag.text.as_str() {
            "-E" | "-n" | "-H" => index += 1,
            "-u" => {
                index += 1;
                if words.get(index).is_some() {
                    index += 1;
                }
            }
            _ => break,
        }
    }
    index
}
