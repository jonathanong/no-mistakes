use super::{word, TokenWithSpan};

/// Keep conditional/loop DDL out of definite live schema history. Policy facts
/// still retain those statements; this tracks control prefixes, not SQL expressions.
#[derive(Default)]
pub(in crate::codebase::postgres::migration) struct Scope(Vec<&'static str>);

impl Scope {
    pub(in crate::codebase::postgres::migration) fn advance(
        &mut self,
        code: &[&TokenWithSpan],
    ) -> bool {
        let mut at = 0;
        let mut pending_loop = false;
        while let Some(token) = code.get(at) {
            if starts_sql(token) {
                break;
            }
            if word(token, "END") {
                let kind = code
                    .get(at + 1)
                    .and_then(|token| control(token))
                    .unwrap_or("BEGIN");
                if let Some(position) = self.0.iter().rposition(|entry| *entry == kind) {
                    self.0.truncate(position);
                }
                at += usize::from(kind != "BEGIN");
            } else if word(token, "FOR") || word(token, "WHILE") {
                self.0.push("LOOP");
                pending_loop = true;
            } else if word(token, "LOOP") && pending_loop {
                pending_loop = false;
            } else if let Some(kind) = control(token) {
                self.0.push(kind);
            }
            at += 1;
        }
        self.0.iter().all(|kind| *kind == "BEGIN")
    }
}

fn control(token: &TokenWithSpan) -> Option<&'static str> {
    ["BEGIN", "IF", "CASE", "LOOP"]
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
