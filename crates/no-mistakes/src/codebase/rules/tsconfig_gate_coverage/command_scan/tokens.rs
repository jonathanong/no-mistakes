pub(crate) fn static_tokens(segment: &str) -> Option<Vec<String>> {
    if segment.is_empty()
        || segment.contains("||")
        || segment.contains('|')
        || segment.contains(['$', '`'])
    {
        return None;
    }
    let mut tokens = Vec::new();
    let mut chars = segment.chars().peekable();
    while chars.peek().is_some() {
        while chars
            .peek()
            .is_some_and(|character| character.is_whitespace())
        {
            chars.next();
        }
        let Some(first) = chars.next() else { break };
        let mut token = String::new();
        if matches!(first, '\'' | '"') {
            let quote = first;
            let mut closed = false;
            for character in chars.by_ref() {
                if character == quote {
                    closed = true;
                    break;
                }
                token.push(character);
            }
            if !closed
                || chars
                    .peek()
                    .is_some_and(|character| !character.is_whitespace())
            {
                return None;
            }
        } else {
            append_unquoted_token(first, &mut chars, &mut token)?;
        }
        if token.is_empty() {
            return None;
        }
        tokens.push(token);
    }
    Some(tokens)
}

fn append_unquoted_token(
    first: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    token: &mut String,
) -> Option<()> {
    let mut character = first;
    loop {
        if character == '\\' {
            token.push(chars.next()?);
        } else {
            token.push(character);
        }
        let Some(next) = chars.next_if(|character| !character.is_whitespace()) else {
            return Some(());
        };
        character = next;
    }
}
