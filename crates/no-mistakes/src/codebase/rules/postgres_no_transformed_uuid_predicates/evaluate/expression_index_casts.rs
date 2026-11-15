use sqlparser::tokenizer::Token;
#[path = "expression_index_cast_operands.rs"]
mod operands;

pub(super) fn normalize(tokens: &mut Vec<Token>) {
    while let Some((start, end, separator)) = innermost(tokens) {
        let primary = operands::is_operand(&tokens[start + 2..separator]);
        let mut rewritten = Vec::new();
        if !primary {
            rewritten.push(Token::LParen);
        }
        rewritten.extend_from_slice(&tokens[start + 2..separator]);
        if !primary {
            rewritten.push(Token::RParen);
        }
        rewritten.push(Token::DoubleColon);
        rewritten.extend_from_slice(&tokens[separator + 1..end]);
        tokens.splice(start..=end, rewritten);
    }
}

fn innermost(tokens: &[Token]) -> Option<(usize, usize, usize)> {
    (0..tokens.len()).find_map(|index| {
        let (end, separator) = cast_bounds(tokens, index)?;
        (!has_cast(&tokens[index + 1..end])).then_some((index, end, separator))
    })
}

fn has_cast(tokens: &[Token]) -> bool {
    (0..tokens.len()).any(|index| cast_bounds(tokens, index).is_some())
}

fn cast_bounds(tokens: &[Token], index: usize) -> Option<(usize, usize)> {
    if !operands::bare(tokens, index, "cast") || (index > 0 && tokens[index - 1] == Token::Period) {
        return None;
    }
    if tokens.get(index + 1) != Some(&Token::LParen) {
        return None;
    }
    let mut depth = 0;
    let mut separator = None;
    for cursor in index + 1..tokens.len() {
        match &tokens[cursor] {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    let separator = separator?;
                    return (index + 2 < separator && separator + 1 < cursor)
                        .then_some((cursor, separator));
                }
            }
            _ if depth == 1
                && operands::bare(tokens, cursor, "as")
                && tokens[cursor - 1] != Token::Period =>
            {
                if separator.is_some() {
                    return None;
                }
                separator = Some(cursor);
            }
            _ => {}
        }
    }
    None
}
