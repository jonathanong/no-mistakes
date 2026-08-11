pub(crate) fn effective_tokens(mut tokens: &[String]) -> Option<&[String]> {
    if tokens
        .first()
        .is_some_and(|token| is_assignment_prefix(token))
    {
        return None;
    }
    loop {
        match tokens.first()?.as_str() {
            "builtin" => return None,
            "command" => {
                tokens = tokens.get(command_target_index(tokens)?..)?;
                if tokens
                    .first()
                    .is_some_and(|token| is_assignment_prefix(token))
                {
                    return None;
                }
            }
            _ => return Some(tokens),
        }
    }
}

pub(super) fn effective_first_word(command: &str) -> Option<&str> {
    let words = command.split_whitespace().collect::<Vec<_>>();
    let mut index = 0;
    if words
        .get(index)
        .is_some_and(|word| is_assignment_prefix(word))
    {
        return None;
    }
    loop {
        match *words.get(index)? {
            "builtin" => return None,
            "command" => {
                index += command_word_target_index(&words[index..])?;
                if words
                    .get(index)
                    .is_some_and(|word| is_assignment_prefix(word))
                {
                    return None;
                }
            }
            word => return Some(word),
        }
    }
}

pub(super) fn has_dynamic_first_word(command: &str) -> bool {
    effective_first_word(command).is_some_and(|word| {
        word.contains(['$', '`'])
            || word.contains(['\'', '"']) && !entirely_literal_quoted_word(word)
    })
}

fn entirely_literal_quoted_word(word: &str) -> bool {
    ['\'', '"'].into_iter().any(|quote| {
        word.strip_prefix(quote)
            .and_then(|word| word.strip_suffix(quote))
            .is_some_and(|word| !word.is_empty() && !word.contains(quote))
    })
}

pub(super) fn has_leading_redirection(tokens: &[String]) -> bool {
    effective_tokens(tokens)
        .and_then(|tokens| tokens.first())
        .is_some_and(|token| {
            token.starts_with(['<', '>'])
                || token.starts_with(char::is_numeric) && token.contains(['<', '>'])
        })
}

fn is_assignment_prefix(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character.is_ascii_alphanumeric()
                    && (index > 0 || character.is_ascii_alphabetic())
        })
}

fn command_target_index(tokens: &[String]) -> Option<usize> {
    match tokens.get(1)?.as_str() {
        "--" => (tokens.len() > 2).then_some(2),
        argument if argument.starts_with('-') => None,
        _ => Some(1),
    }
}

fn command_word_target_index(words: &[&str]) -> Option<usize> {
    match *words.get(1)? {
        "--" => (words.len() > 2).then_some(2),
        argument if argument.starts_with('-') => None,
        _ => Some(1),
    }
}
