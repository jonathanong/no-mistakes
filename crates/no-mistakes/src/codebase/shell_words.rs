#[derive(Clone, Copy)]
pub(crate) enum Whitespace {
    Ascii,
    Unicode,
}

pub(crate) fn split_quoted_words(input: &str, whitespace: Whitespace) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                current.push(character);
            }
            continue;
        }
        let is_whitespace = match whitespace {
            Whitespace::Ascii => character.is_ascii_whitespace(),
            Whitespace::Unicode => character.is_whitespace(),
        };
        if is_whitespace && quote.is_none() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }
    if escaped || quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        words.push(current);
    }
    Some(words)
}

#[cfg(test)]
#[path = "shell_words_tests.rs"]
mod tests;
