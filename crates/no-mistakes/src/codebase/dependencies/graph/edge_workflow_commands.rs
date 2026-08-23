fn static_command_segments(input: &str) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        if escaped {
            current.push(character);
            escaped = false;
            index += 1;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            current.push(character);
            index += 1;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            current.push(character);
            index += 1;
            continue;
        }
        let comment = quote.is_none()
            && character == '#'
            && (current.is_empty()
                || current
                    .chars()
                    .last()
                    .is_some_and(char::is_whitespace));
        if comment {
            if push_static_segment(&mut segments, &current) {
                return segments;
            }
            current.clear();
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            index += usize::from(index < chars.len());
            continue;
        }
        let paired_separator = quote.is_none()
            && matches!(character, '&' | '|')
            && chars.get(index + 1) == Some(&character);
        let separator = quote.is_none() && (character == '\n' || character == ';');
        if paired_separator || separator {
            if push_static_segment(&mut segments, &current) {
                return segments;
            }
            current.clear();
            index += usize::from(paired_separator) + 1;
            continue;
        }
        current.push(character);
        index += 1;
    }
    push_static_segment(&mut segments, &current);
    segments
}

/// Returns true when later commands share an unknown changed directory.
fn push_static_segment(segments: &mut Vec<Vec<String>>, segment: &str) -> bool {
    let segment = segment.trim();
    if segment.is_empty()
        || segment.contains('|')
        || segment.contains("$(")
        || segment.contains('`')
    {
        return false;
    }
    if let Some(words) = shellish_literal_words(segment) {
        if words
            .iter()
            .find(|word| !is_environment_assignment(word))
            .is_some_and(|word| word == "cd")
        {
            return true;
        }
        if !words.is_empty() {
            segments.push(words);
        }
    }
    false
}

fn shellish_literal_words(input: &str) -> Option<Vec<String>> {
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
        if character.is_whitespace() && quote.is_none() {
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

fn is_static_path_token(token: &str) -> bool {
    !token.is_empty()
        && !token
            .chars()
            .any(|character| matches!(character, '$' | '`' | '*' | '?' | '[' | ']' | '{' | '}'))
}

fn is_environment_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

include!("edge_workflow_command_targets.rs");
