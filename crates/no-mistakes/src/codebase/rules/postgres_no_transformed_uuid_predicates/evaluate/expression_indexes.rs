use super::Columns;
use crate::codebase::postgres::statements::SqlTransformedColumnUseFact;
use sqlparser::ast::{Expr, VisitMut, VisitorMut};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};
use std::ops::ControlFlow;

pub(super) type IndexCache =
    dashmap::DashMap<(usize, String), (std::sync::Arc<Expr>, bool), rustc_hash::FxBuildHasher>;

impl Columns<'_> {
    pub(super) fn expression_indexed(
        &self,
        use_: &SqlTransformedColumnUseFact,
        table: &str,
    ) -> bool {
        if !use_.index_operand || !use_.expression_single_relation {
            return false;
        }
        let key = (
            std::sync::Arc::as_ptr(&use_.expression) as usize,
            table.to_string(),
        );
        self.expression_indexes
            .entry(key)
            .or_insert_with(|| {
                (
                    use_.expression.clone(),
                    self.compute_expression_index(use_, table),
                )
            })
            .value()
            .1
    }
    fn compute_expression_index(&self, use_: &SqlTransformedColumnUseFact, table: &str) -> bool {
        if use_.expression_relations.is_empty()
            || !use_.expression_relations.iter().all(|reference| {
                self.resolve_owner(
                    &reference.table,
                    &reference.column,
                    reference.candidate_tables.as_deref(),
                )
                .is_ok_and(|(owner, _)| owner == table)
            })
        {
            return false;
        }
        let Some(table) = self.catalog.bounded_relation(table) else {
            return false;
        };
        let mut expression = (*use_.expression).clone();
        if VisitMut::visit(&mut expression, &mut Unqualify).is_break() {
            return false;
        }
        let Some(expression) = tokens(&expression.to_string()) else {
            return false;
        };
        table.indexes.iter().any(|index| {
            index.valid
                && index.ready
                && index.live
                && index.predicate.is_none()
                && index.access_method.eq_ignore_ascii_case("btree")
                && index.keys.first().is_some_and(|key| {
                    key.ordering_supported
                        && key.column.is_none()
                        && tokens(&key.expression).as_ref() == Some(&expression)
                })
        })
    }
}

struct Unqualify;
impl VisitorMut for Unqualify {
    type Break = ();
    fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        match expr {
            Expr::CompoundIdentifier(parts) => {
                *expr = Expr::Identifier(parts.last().expect("compound identifier").clone());
            }
            Expr::Subquery(_) | Expr::Exists { .. } | Expr::InSubquery { .. } => {
                return ControlFlow::Break(());
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

fn tokens(expression: &str) -> Option<Vec<Token>> {
    let mut tokens: Vec<_> = Tokenizer::new(&PostgreSqlDialect {}, expression)
        .tokenize()
        .ok()?
        .into_iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .map(|token| match token {
            Token::Word(mut word) if word.quote_style.is_none() => {
                word.value.make_ascii_lowercase();
                Token::Word(word)
            }
            other => other,
        })
        .collect();
    normalize_rows(&mut tokens);
    while matches!(tokens.first(), Some(Token::LParen)) {
        let mut depth = 0;
        let enclosing = tokens.iter().enumerate().all(|(index, token)| {
            match token {
                Token::LParen => depth += 1,
                Token::RParen => depth -= 1,
                _ => {}
            }
            depth > 0 || index == tokens.len() - 1
        });
        if !enclosing || depth != 0 || tokens.last() != Some(&Token::RParen) {
            break;
        }
        tokens.pop();
        tokens.remove(0);
    }
    Some(tokens)
}

fn normalize_rows(tokens: &mut Vec<Token>) {
    let mut parentheses = Vec::new();
    let mut remove = vec![false; tokens.len()];
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::LParen => parentheses.push((index, false)),
            Token::Comma => {
                if let Some((_, comma)) = parentheses.last_mut() {
                    *comma = true;
                }
            }
            Token::RParen => {
                let Some((open, true)) = parentheses.pop() else {
                    continue;
                };
                if open > 0
                    && matches!(&tokens[open - 1], Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("row"))
                    && (open < 2 || tokens[open - 2] != Token::Period)
                {
                    remove[open - 1] = true;
                }
            }
            _ => {}
        }
    }
    let mut index = 0;
    tokens.retain(|_| {
        let keep = !remove[index];
        index += 1;
        keep
    });
}
