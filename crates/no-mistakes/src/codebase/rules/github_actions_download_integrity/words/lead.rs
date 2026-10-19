use super::{basename, is_assignment, Word};

const LEADING: &[&str] = &[
    "!", "{", "(", "if", "elif", "then", "else", "do", "while", "until", "time", "exec", "command",
    "nohup",
];

pub(super) fn command_at(words: &[Word]) -> Option<(String, usize)> {
    let mut index = 0;
    while index < words.len() {
        let text = unwrap_group(&words[index].text);
        if text.is_empty()
            || is_assignment(&words[index].text)
            || is_assignment(text)
            || LEADING.contains(&text)
        {
            index += 1;
            continue;
        }
        if text == "sudo" || text == "xargs" {
            index = skip_flags(words, index + 1, text == "sudo");
            continue;
        }
        if text == "env" {
            index = skip_env(words, index + 1);
            continue;
        }
        return Some((basename(text), index));
    }
    None
}

fn unwrap_group(text: &str) -> &str {
    text.trim_start_matches(['(', '{'])
        .trim_end_matches([')', '}'])
}

fn skip_flags(words: &[Word], mut index: usize, sudo: bool) -> usize {
    while let Some(word) = words.get(index) {
        match word.text.as_str() {
            "-E" | "-n" | "-H" | "-i" => index += 1,
            "-u" if sudo => {
                index += 1;
                if words.get(index).is_some() {
                    index += 1;
                }
            }
            text if text.starts_with('-') || is_assignment(text) => index += 1,
            _ => break,
        }
    }
    index
}

fn skip_env(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        if word.text == "-i"
            || word.text == "-0"
            || is_assignment(&word.text)
            || word.text.starts_with('-')
        {
            index += 1;
            continue;
        }
        break;
    }
    index
}
