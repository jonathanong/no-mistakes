use super::{FxHashMap, Site, SqlStatementFileFacts, SqlVariantLocations, Token, TokenWithSpan};

pub(super) fn collect(
    facts: &SqlStatementFileFacts,
    tokens: &[TokenWithSpan],
    out: &mut SqlVariantLocations,
) {
    let code: Vec<_> = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect();
    let candidates: Vec<_> = code
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let (name, origin) = parameter(&code, index)?;
            Some((name, token.span.start.line as usize, origin))
        })
        .collect();
    let mut occurrences = FxHashMap::default();
    for (index, setting) in facts.setting_uses.iter().enumerate() {
        let occurrence = occurrences
            .entry((&setting.name, setting.line))
            .or_insert(0usize);
        let at = candidates
            .iter()
            .filter(|(name, line, _)| name == &setting.name && *line == setting.line)
            .nth(*occurrence)
            .map_or((setting.line, 1), |(_, _, origin)| *origin);
        *occurrence += 1;
        out.insert(Site::Setting(index), at.0, at.1);
    }
}

fn parameter(code: &[&TokenWithSpan], index: usize) -> Option<(String, (usize, usize))> {
    let token = code[index];
    if builtin(token, "set_config") {
        if index >= 1
            && code[index - 1].token == Token::Period
            && (index < 2 || !builtin(code[index - 2], "pg_catalog"))
        {
            return None;
        }
        if code.get(index + 1)?.token != Token::LParen || code.get(index + 3)?.token != Token::Comma
        {
            return None;
        }
        let argument = code.get(index + 2)?;
        let name = match &argument.token {
            Token::SingleQuotedString(value) | Token::EscapedStringLiteral(value) => value.clone(),
            Token::DollarQuotedString(value) => value.value.clone(),
            _ => return None,
        };
        return Some((name.to_ascii_lowercase(), at(argument)));
    }
    if !word(token, "SET") || !setting_header(code, index) {
        return None;
    }
    let mut index = index + 1;
    if code
        .get(index)
        .is_some_and(|token| word(token, "LOCAL") || word(token, "SESSION"))
    {
        index += 1;
    }
    let origin = code.get(index)?;
    let Token::Word(name) = &origin.token else {
        return None;
    };
    let mut name =
        if word(origin, "TIME") && code.get(index + 1).is_some_and(|token| word(token, "ZONE")) {
            "timezone".into()
        } else {
            name.value.to_ascii_lowercase()
        };
    while code
        .get(index + 1)
        .is_some_and(|token| token.token == Token::Period)
    {
        let Token::Word(part) = &code.get(index + 2)?.token else {
            break;
        };
        name.push('.');
        name.push_str(&part.value.to_ascii_lowercase());
        index += 2;
    }
    Some((name, at(origin)))
}
fn setting_header(code: &[&TokenWithSpan], index: usize) -> bool {
    let start = code[..index]
        .iter()
        .rposition(|token| token.token == Token::SemiColon)
        .map_or(0, |position| position + 1);
    index == start
        || (word(code[start], "ALTER")
            && code
                .get(start + 1)
                .is_some_and(|token| word(token, "DATABASE") || word(token, "SYSTEM")))
}
fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.value.eq_ignore_ascii_case(expected))
}
fn builtin(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if if word.quote_style.is_some() { word.value == expected } else { word.value.eq_ignore_ascii_case(expected) })
}
fn at(token: &TokenWithSpan) -> (usize, usize) {
    (
        token.span.start.line as usize,
        token.span.start.column as usize,
    )
}
