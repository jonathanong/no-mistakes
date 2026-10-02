use super::{word, Token, TokenWithSpan};

#[cfg(test)]
#[path = "execution/tests.rs"]
mod tests;

/// Keep conditional/loop DDL out of definite live schema history. Policy facts
/// still retain those statements; this tracks control prefixes, not SQL expressions.
#[derive(Default)]
pub(in crate::codebase::postgres::migration) struct Scope {
    blocks: Vec<&'static str>,
    // A plain RETURN makes the remaining body non-definite, including after a branch closes.
    execution_stopped: bool,
}

impl Scope {
    pub(in crate::codebase::postgres::migration) fn advance(
        &mut self,
        code: &[&TokenWithSpan],
    ) -> bool {
        let was_stopped = self.execution_stopped;
        let mut at = 0;
        let mut pending_loop = false;
        while let Some(token) = code.get(at) {
            if unquoted_word(token, "RETURN") {
                let continues = code.get(at + 1).is_some_and(|next| {
                    unquoted_word(next, "NEXT") || unquoted_word(next, "QUERY")
                });
                if !continues {
                    self.execution_stopped = true;
                }
                break;
            }
            if starts_sql(token) {
                break;
            }
            if word(token, "END") {
                let kind = code
                    .get(at + 1)
                    .and_then(|token| control(token))
                    .unwrap_or("BEGIN");
                if let Some(position) = self.blocks.iter().rposition(|entry| *entry == kind) {
                    self.blocks.truncate(position);
                }
                at += usize::from(kind != "BEGIN");
            } else if word(token, "FOR") || word(token, "WHILE") {
                self.blocks.push("LOOP");
                pending_loop = true;
            } else if word(token, "LOOP") && pending_loop {
                pending_loop = false;
            } else if let Some(kind) = control(token) {
                self.blocks.push(kind);
            }
            at += 1;
        }
        !was_stopped && self.blocks.iter().all(|kind| *kind == "BEGIN")
    }
}

fn unquoted_word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn control(token: &TokenWithSpan) -> Option<&'static str> {
    ["BEGIN", "IF", "CASE", "LOOP", "EXCEPTION"]
        .into_iter()
        .find(|kind| word(token, kind))
}

fn starts_sql(token: &TokenWithSpan) -> bool {
    [
        "CREATE", "ALTER", "DROP", "EXECUTE", "SELECT", "INSERT", "UPDATE", "DELETE", "RETURN",
    ]
    .into_iter()
    .any(|kind| word(token, kind))
        || matches!(token.token, super::Token::Assignment | super::Token::Eq)
}
