use super::super::fragment_ast::FragmentStar;
use super::clauses::OpenClause;
use super::context::extract_context_from_tokens;
use super::readers::{
    is_bare_star_prev, read_qualified_name_backwards, split_top_level_statements,
};
use super::token_scopes::{
    enclosing_function, is_cte_relation, is_known_cte_or_derived, resolve_qualified_relation,
    scope_context,
};
use super::{is_kw, FragmentContext};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn extract_stars(
    tokens: &[TokenWithSpan],
    initial: OpenClause,
    out: &mut Vec<FragmentStar>,
) {
    for (idx, stmt) in split_top_level_statements(tokens).into_iter().enumerate() {
        let open = if idx == 0 { initial } else { OpenClause::default() };
        extract_statement_stars(stmt, open, out);
    }
}

fn extract_statement_stars(
    tokens: &[TokenWithSpan],
    initial: OpenClause,
    out: &mut Vec<FragmentStar>,
) {
    let mut ctx = FragmentContext::default();
    extract_context_from_tokens(tokens, &mut ctx);
    let (mut clause_select, mut clause_returning, mut in_exists) =
        (initial.select, initial.returning, initial.in_exists);
    let mut clause_stack: Vec<(bool, bool, bool)> = Vec::new();
    let (mut prev_code, mut prev_prev_idx): (Option<(usize, &TokenWithSpan)>, Option<usize>) =
        (None, None);
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        match &token.token {
            Token::LParen => {
                clause_stack.push((clause_select, clause_returning, in_exists));
                in_exists = prev_code.is_some_and(|(_, t)| is_kw(t, Keyword::EXISTS));
            }
            Token::RParen => {
                if let Some((prev_select, prev_returning, prev_exists)) = clause_stack.pop() {
                    clause_select = prev_select;
                    clause_returning = prev_returning;
                    in_exists = prev_exists;
                }
            }
            Token::Mul if !in_exists && (clause_returning || clause_select) => {
                let line = (token.span.start.line as usize).max(1);
                let scoped = scope_context(tokens, i, &ctx);
                let active = scoped.as_ref().unwrap_or(&ctx);
                if prev_code.is_some_and(|(_, t)| matches!(t.token, Token::Period)) {
                    push_qualified(
                        tokens,
                        i,
                        prev_prev_idx,
                        active,
                        clause_returning,
                        line,
                        out,
                    );
                } else if (prev_code.is_none() && initial.expects_expr)
                    || prev_code.is_some_and(|(pi, t)| is_bare_star_prev(tokens, pi, t))
                {
                    push_bare(tokens, i, active, clause_returning, line, out);
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
        prev_prev_idx = prev_code.map(|(idx, _)| idx);
        prev_code = Some((i, token));
    }
}

fn push_qualified(
    tokens: &[TokenWithSpan],
    index: usize,
    prev_prev_idx: Option<usize>,
    ctx: &FragmentContext,
    returning: bool,
    line: usize,
    out: &mut Vec<FragmentStar>,
) {
    let qualifier = prev_prev_idx.and_then(|idx| read_qualified_name_backwards(tokens, idx));
    let func = enclosing_function(tokens, index);
    if let Some(q) = qualifier.as_deref() {
        let joined: Vec<_> = ctx
            .join_aliases
            .iter()
            .filter(|(a, _)| a == q)
            .map(|(_, t)| t.clone())
            .collect();
        if !joined.is_empty() {
            for rel in joined {
                if is_cte_relation(ctx, &rel) {
                    continue;
                }
                out.push(FragmentStar {
                    returning,
                    qualifier: qualifier.clone(),
                    relation: Some(rel),
                    within_function: func.clone(),
                    line,
                });
            }
            return;
        }
    }
    let resolved = qualifier
        .as_deref()
        .and_then(|q| resolve_qualified_relation(ctx, q, returning));
    if let Some((rel, is_physical_target)) = &resolved {
        if !is_physical_target && is_cte_relation(ctx, rel) {
            return;
        }
    } else if qualifier
        .as_deref()
        .is_some_and(|q| is_known_cte_or_derived(ctx, q))
    {
        return;
    }
    out.push(FragmentStar {
        returning,
        qualifier,
        relation: resolved.map(|(rel, _)| rel),
        within_function: func,
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
    if func.as_deref().is_some_and(|f| {
        f.eq_ignore_ascii_case("count") || f.eq_ignore_ascii_case("pg_catalog.count")
    }) {
        return;
    }
    if returning {
        out.push(FragmentStar {
            returning: true,
            qualifier: None,
            relation: ctx.mutation_target.as_ref().map(|(t, _)| t.clone()),
            within_function: func,
            line,
        });
        return;
    }
    if ctx.from_relations.is_empty() {
        if ctx.has_derived_from {
            return;
        }
        out.push(FragmentStar {
            returning: false,
            qualifier: None,
            relation: None,
            within_function: func,
            line,
        });
        return;
    }
    for (t, _) in &ctx.from_relations {
        if is_cte_relation(ctx, t) {
            continue;
        }
        out.push(FragmentStar {
            returning: false,
            qualifier: None,
            relation: Some(t.clone()),
            within_function: func.clone(),
            line,
        });
    }
}
