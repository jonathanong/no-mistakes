use super::{identifier, DynamicSql, TokenWithSpan};
use std::collections::HashMap;

pub(super) type Variables = HashMap<String, Option<DynamicSql>>;

/// PL/pgSQL block-local variable bindings: a nested `DECLARE` shadows an outer
/// name only until its block's `END`, then the outer binding is restored.
#[derive(Default)]
pub(super) struct Blocks {
    // Outer binding saved the first time a block declares a name (None = unbound).
    frames: Vec<HashMap<String, Option<Option<DynamicSql>>>>,
    declaring: bool,
}

impl Blocks {
    pub(super) fn enter(&mut self, code: &[&TokenWithSpan], variables: &mut Variables) {
        // A statement chunk starts with `;`-free keyword chains such as `BEGIN DECLARE x ...`.
        let mut at = 0;
        while let Some(token) = code.get(at) {
            if keyword(token, "DECLARE") {
                self.open();
                self.declaring = true;
            } else if keyword(token, "BEGIN") {
                if self.declaring {
                    self.declaring = false;
                } else {
                    self.open();
                }
            } else {
                break;
            }
            at += 1;
        }
        let rest = &code[at..];
        let Some(first) = rest.first() else { return };
        if self.declaring {
            self.declare(rest, variables);
        } else if keyword(first, "END")
            && !rest.get(1).is_some_and(|next| {
                ["IF", "LOOP", "CASE"]
                    .into_iter()
                    .any(|kind| keyword(next, kind))
            })
        {
            self.close(variables);
        }
    }

    fn open(&mut self) {
        self.frames.push(HashMap::new());
    }

    fn declare(&mut self, code: &[&TokenWithSpan], variables: &mut Variables) {
        let Some(name) = code.first().and_then(|token| identifier(token)) else {
            return;
        };
        let name = name.to_ascii_lowercase();
        if let Some(frame) = self.frames.last_mut() {
            frame
                .entry(name.clone())
                .or_insert_with(|| variables.get(&name).cloned());
        }
        // An uninitialized declaration is NULL until assigned; an initializer overwrites this.
        variables.insert(name, None);
    }

    fn close(&mut self, variables: &mut Variables) {
        for (name, outer) in self.frames.pop().unwrap_or_default() {
            match outer {
                Some(value) => variables.insert(name, value),
                None => variables.remove(&name),
            };
        }
    }
}

fn keyword(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, super::Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
