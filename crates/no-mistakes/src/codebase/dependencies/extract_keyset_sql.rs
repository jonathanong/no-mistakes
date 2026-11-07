fn keyset_literal_sql(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Some(
            template
                .quasis
                .iter()
                .map(|part| crate::ast::template_quasi_text(part))
                .collect::<String>(),
        ),
        _ => None,
    }
}

fn keyset_sql(sql: &str, key: &str) -> bool {
    use sqlparser::tokenizer::{Token, Tokenizer};
    let Ok(tokens) = Tokenizer::new(&sqlparser::dialect::PostgreSqlDialect {}, sql).tokenize()
    else {
        return false;
    };
    let mut tokens = tokens
        .into_iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    if tokens.last() == Some(&Token::SemiColon) {
        tokens.pop();
    }
    let Some(where_at) = tokens.iter().position(|token| keyset_word(token, "WHERE")) else {
        return false;
    };
    if !keyset_projection(&tokens[..where_at], key) {
        return false;
    }
    let tail = &tokens[where_at..];
    let (limit_at, count_at) = match tail.len() {
        10 if keyset_word(&tail[7], "ASC") => (8, 9),
        9 => (7, 8),
        _ => return false,
    };
    keyset_word(&tail[0], "WHERE")
        && keyset_identifier(&tail[1]).as_deref() == Some(key)
        && tail[2] == Token::Gt
        && tail[3] == Token::Placeholder("$1".into())
        && keyset_word(&tail[4], "ORDER")
        && keyset_word(&tail[5], "BY")
        && keyset_identifier(&tail[6]).as_deref() == Some(key)
        && keyset_word(&tail[limit_at], "LIMIT")
        && matches!(&tail[count_at], Token::Number(value, false) if value.parse::<u64>().is_ok_and(|limit| limit > 1))
}

fn keyset_word(token: &sqlparser::tokenizer::Token, expected: &str) -> bool {
    matches!(token, sqlparser::tokenizer::Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn keyset_identifier(token: &sqlparser::tokenizer::Token) -> Option<String> {
    let sqlparser::tokenizer::Token::Word(word) = token else {
        return None;
    };
    match word.quote_style {
        None => Some(word.value.to_ascii_lowercase()),
        Some('"') => Some(word.value.clone()),
        _ => None,
    }
}

fn keyset_projection(prefix: &[sqlparser::tokenizer::Token], key: &str) -> bool {
    use sqlparser::tokenizer::Token;
    if !prefix
        .first()
        .is_some_and(|token| keyset_word(token, "SELECT"))
    {
        return false;
    }
    let Some(from_at) = prefix.iter().position(|token| keyset_word(token, "FROM")) else {
        return false;
    };
    let source = &prefix[from_at + 1..];
    let (table, consumed) = keyset_path(source);
    if table.is_empty() || table.len() > 2 {
        return false;
    }
    let alias = match &source[consumed..] {
        [] => None,
        [name] => keyset_identifier(name),
        [keyword, name] if keyset_word(keyword, "AS") => keyset_identifier(name),
        _ => return false,
    };
    if consumed != source.len() && alias.is_none() {
        return false;
    }
    let qualifier = alias.map_or_else(|| table.clone(), |name| vec![name]);
    let mut selected_key = false;
    for projection in prefix[1..from_at].split(|token| *token == Token::Comma) {
        let (column, consumed) = keyset_path(projection);
        let rest = &projection[consumed..];
        let wildcard = rest == [Token::Mul] && column.is_empty()
            || rest == [Token::Period, Token::Mul] && keyset_qualifier(&column, &qualifier);
        if wildcard {
            selected_key = true;
            continue;
        }
        let Some((name, column_qualifier)) = column.split_last() else {
            return false;
        };
        if !column_qualifier.is_empty() && !keyset_qualifier(column_qualifier, &qualifier) {
            return false;
        }
        let output = match rest {
            [] => name.clone(),
            [alias] => match keyset_identifier(alias) {
                Some(alias) => alias,
                None => return false,
            },
            [keyword, alias] if keyset_word(keyword, "AS") => match keyset_identifier(alias) {
                Some(alias) => alias,
                None => return false,
            },
            _ => return false,
        };
        if output == key {
            if name != key {
                return false;
            }
            selected_key = true;
        }
    }
    selected_key
}

fn keyset_qualifier(path: &[String], table: &[String]) -> bool {
    path == table || path.len() == 1 && path.last() == table.last()
}

fn keyset_path(tokens: &[sqlparser::tokenizer::Token]) -> (Vec<String>, usize) {
    use sqlparser::tokenizer::Token;
    let mut path = Vec::new();
    let mut at = 0;
    while let Some(token) = tokens.get(at) {
        let Some(name) = keyset_identifier(token) else {
            break;
        };
        path.push(name);
        at += 1;
        if tokens.get(at) != Some(&Token::Period)
            || !tokens
                .get(at + 1)
                .is_some_and(|token| keyset_identifier(token).is_some())
        {
            break;
        }
        at += 1;
    }
    (path, at)
}
