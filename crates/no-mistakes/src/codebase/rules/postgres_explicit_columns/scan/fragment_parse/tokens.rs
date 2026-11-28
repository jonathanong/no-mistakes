use super::super::fragment_ast::FragmentStar;
use super::context::extract_context_from_tokens;
use super::{is_kw, word_value, FragmentContext};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn extract_stars(
    tokens: &[TokenWithSpan],
    initial_select: bool,
    initial_returning: bool,
    out: &mut Vec<FragmentStar>,
) {
    let mut ctx = FragmentContext::default();
    extract_context_from_tokens(tokens, &mut ctx);
    let mut clause_returning = initial_returning;
    let mut clause_select = initial_select;
    let mut clause_stack: Vec<(bool, bool)> = Vec::new();
    let mut in_exists = false;
    let mut exists_depth = 0usize;
    let mut depth = 0usize;
    let mut prev_code: Option<&TokenWithSpan> = None;
    let mut prev_prev: Option<&TokenWithSpan> = None;
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        match &token.token {
            Token::LParen => {
                clause_stack.push((clause_select, clause_returning));
                depth += 1;
                if prev_code.is_some_and(|t| is_kw(t, Keyword::EXISTS)) && !in_exists {
                    in_exists = true;
                    exists_depth = depth;
                }
            }
            Token::RParen => {
                if in_exists && depth == exists_depth {
                    in_exists = false;
                }
                depth = depth.saturating_sub(1);
                if let Some((prev_select, prev_returning)) = clause_stack.pop() {
                    clause_select = prev_select;
                    clause_returning = prev_returning;
                }
            }
            Token::Mul if !in_exists && (clause_returning || clause_select) => {
                let line = (token.span.start.line as usize).max(1);
                if prev_code.is_some_and(|t| matches!(t.token, Token::Period)) {
                    push_qualified(tokens, i, prev_prev, &ctx, clause_returning, line, out);
                } else if prev_code.is_none()
                    || prev_code.is_some_and(|t| {
                        matches!(t.token, Token::Comma | Token::LParen)
                            || is_kw(t, Keyword::SELECT)
                            || is_kw(t, Keyword::DISTINCT)
                            || is_kw(t, Keyword::RETURNING)
                    })
                {
                    push_bare(tokens, i, &ctx, clause_returning, line, out);
                }
            }
            _ => {
                if is_kw(token, Keyword::RETURNING) {
                    clause_returning = true;
                    clause_select = false;
                } else if is_kw(token, Keyword::SELECT) {
                    clause_select = true;
                    clause_returning = false;
                } else if is_kw(token, Keyword::FROM)
                    || is_kw(token, Keyword::WHERE)
                    || is_kw(token, Keyword::INTO)
                {
                    clause_select = false;
                    clause_returning = false;
                }
            }
        }
        prev_prev = prev_code;
        prev_code = Some(token);
    }
}

fn push_qualified(
    tokens: &[TokenWithSpan],
    index: usize,
    prev_prev: Option<&TokenWithSpan>,
    ctx: &FragmentContext,
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let qualifier = prev_prev.and_then(word_value);
    let relation = qualifier.as_deref().and_then(|q| {
        ctx.from_relations
            .iter()
            .chain(ctx.mutation_target.iter())
            .find(|(_, a)| a.as_deref() == Some(q))
            .or_else(|| {
                ctx.from_relations
                    .iter()
                    .chain(ctx.mutation_target.iter())
                    .find(|(t, a)| {
                        a.is_none() && (t == q || t.rsplit('.').next() == Some(q))
                    })
            })
            .map(|(t, _)| t.clone())
    });
    if let Some(rel) = &relation {
        if !rel.contains('.') && ctx.ctes.iter().any(|c| c.eq_ignore_ascii_case(rel)) {
            return;
        }
    } else if qualifier
        .as_deref()
        .is_some_and(|q| ctx.ctes.iter().any(|c| c.eq_ignore_ascii_case(q)))
    {
        return;
    }
    out.push(FragmentStar {
        returning,
        qualifier,
        relation,
        within_function: enclosing_function(tokens, index),
        line,
    });
}

fn push_bare(
    tokens: &[TokenWithSpan],
    index: usize,
    ctx: &FragmentContext,
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let func = enclosing_function(tokens, index);
    if func.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("count")) {
        return;
    }
    let relation = if returning {
        ctx.mutation_target.as_ref().map(|(t, _)| t.clone())
    } else if ctx.from_relations.len() == 1 {
        let (t, _) = &ctx.from_relations[0];
        if ctx.ctes.iter().any(|c| c.eq_ignore_ascii_case(t)) {
            return;
        }
        Some(t.clone())
    } else {
        None
    };
    out.push(FragmentStar {
        returning,
        qualifier: None,
        relation,
        within_function: func,
        line,
    });
}

fn enclosing_function(tokens: &[TokenWithSpan], star_index: usize) -> Option<String> {
    let mut depth = 0usize;
    for i in (0..star_index).rev() {
        match &tokens[i].token {
            Token::RParen => depth += 1,
            Token::LParen => {
                if depth == 0 {
                    for j in (0..i).rev() {
                        if matches!(tokens[j].token, Token::Whitespace(_)) {
                            continue;
                        }
                        return word_value(&tokens[j]);
                    }
                    return None;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}
