use super::super::sql_text::SqlText;
use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::{
    helpers::program::{Expr, Mark},
    walk::resolve::functions::provenance::RecoveredSql,
};

impl Engine<'_> {
    pub(super) fn template(
        &mut self,
        pieces: &[RecoveredSql],
        holes: &[(Expr, Mark)],
        state: State,
        binds: bool,
    ) -> Option<Vec<(State, Value)>> {
        if pieces.len() != holes.len() + 1 {
            return None;
        }
        let file = self.file()?.to_path_buf();
        let expressions = holes.iter().map(|(expr, _)| expr).collect::<Vec<_>>();
        let mut out = Vec::new();
        'states: for (mut state, values) in self.all(&expressions, state)? {
            let mut text = SqlText::literal(pieces.first()?.clone().with_source_file(&file));
            if self.sql_bytes(text.raw.sql.len()).is_none() {
                self.failures += 1;
                continue;
            }
            for (index, (value, (_, mark))) in values.into_iter().zip(holes).enumerate() {
                let part = if binds {
                    SqlText::bind(*mark, &file, state.contains_executor(&value))
                } else {
                    let Value::Text(text) = value else {
                        self.failures += 1;
                        continue 'states;
                    };
                    text
                };
                let Some(bytes) = text.raw.sql.len().checked_add(part.raw.sql.len()) else {
                    self.failures += 1;
                    continue 'states;
                };
                if self.sql_bytes(bytes).is_none() || text.append(part).is_none() {
                    self.failures += 1;
                    continue 'states;
                }
                let part = SqlText::literal(pieces[index + 1].clone().with_source_file(&file));
                let Some(bytes) = text.raw.sql.len().checked_add(part.raw.sql.len()) else {
                    self.failures += 1;
                    continue 'states;
                };
                if self.sql_bytes(bytes).is_none() || text.append(part).is_none() {
                    self.failures += 1;
                    continue 'states;
                }
            }
            let value = if binds {
                state.allocate(text)
            } else {
                Value::Text(text)
            };
            out.push((state, value));
        }
        Some(out)
    }
}
