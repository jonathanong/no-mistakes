use super::super::lower::Lowered;
use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Stmt;
use std::path::Path;

impl Engine<'_> {
    pub fn evaluate_at(
        &mut self,
        file: &Path,
        lowered: &Lowered,
        site: Option<super::super::Span>,
    ) -> Option<Vec<Value>> {
        self.evaluate_rows(file, lowered, site, None)
    }

    pub fn evaluate_iteration(
        &mut self,
        file: &Path,
        lowered: &Lowered,
        site: super::super::Span,
        parameter: &str,
        rows: &[crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql],
    ) -> Option<Vec<Value>> {
        self.evaluate_rows(file, lowered, Some(site), Some((parameter, rows)))
    }

    fn evaluate_rows(
        &mut self,
        file: &Path,
        lowered: &Lowered,
        site: Option<super::super::Span>,
        rows: Option<(&str, &[crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql])>,
    ) -> Option<Vec<Value>> {
        self.root_file = Some(file.to_path_buf());
        self.site = site;
        let mut states = match rows {
            None => vec![State::default()],
            Some((name, rows)) => rows
                .iter()
                .map(|row| {
                    let mut state = State::default();
                    let text = super::super::sql_text::SqlText::literal(
                        row.clone().with_source_file(file),
                    );
                    state.env.insert(name.to_string(), Value::Text(text));
                    state
                })
                .collect(),
        };
        self.cap(&mut states);
        for state in &mut states {
            for (id, _) in &lowered.references {
                if site.is_none_or(|site| self.catalog.local(file, *id, site).is_none()) {
                    state
                        .env
                        .entry(format!("binding:{id}"))
                        .or_insert(Value::Scalar);
                }
            }
        }
        let mut values = Vec::new();
        for state in self.block(&lowered.program.body, states)? {
            let Some(value) = state.materialize(state.returned.clone().unwrap_or(Value::Undefined))
            else {
                self.failures += 1;
                continue;
            };
            if !values.contains(&value) {
                values.push(value);
            }
        }
        (!values.is_empty()).then_some(values)
    }
    /// Evaluate a guarded initializer in the current row and heap state.
    pub(super) fn local_value(
        &mut self,
        id: usize,
        mut state: State,
    ) -> Option<Vec<(State, Value)>> {
        let file = self.file()?.to_path_buf();
        let site = self.site?;
        if self.local_stack.len() >= 8 || self.local_stack.contains(&(file.clone(), id)) {
            return None;
        }
        let local = self.catalog.local(&file, id, site)?.clone();
        for (reference, _) in &local.references {
            if self.catalog.local(&file, *reference, site).is_none() {
                state
                    .env
                    .entry(format!("binding:{reference}"))
                    .or_insert(Value::Scalar);
            }
        }
        let [Stmt::Return(Some(expr), _)] = local.program.body.as_slice() else {
            return None;
        };
        self.charge()?;
        self.local_stack.push((file, id));
        let result = self.expression(expr, state);
        self.local_stack.pop();
        result
    }
}
