use super::super::sql_text::SqlText;
use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::{Entry, Expr};

impl Engine<'_> {
    pub(super) fn expression(&mut self, expr: &Expr, state: State) -> Option<Vec<(State, Value)>> {
        match self.expression_inner(expr, state) {
            Some(values) => Some(values),
            None => {
                self.failures += 1;
                Some(Vec::new())
            }
        }
    }
    fn expression_inner(&mut self, expr: &Expr, state: State) -> Option<Vec<(State, Value)>> {
        self.charge()?;
        if let Expr::Lit(text) = expr {
            self.sql_bytes(text.sql.len())?;
        }
        let values = match expr {
            Expr::Lit(text) => vec![(
                state,
                Value::Text(SqlText::literal(
                    text.clone().with_source_file(self.file()?),
                )),
            )],
            Expr::Bool(value) => vec![(state, Value::Bool(*value))],
            Expr::Null => vec![(state, Value::Null)],
            Expr::Undefined => vec![(state, Value::Undefined)],
            Expr::Num => vec![(state, Value::Scalar)],
            Expr::Ident(name) => self.identifier_value(name, state)?,
            Expr::Member(object, name) => self.member(object, name, state)?,
            Expr::ComputedMember(object, key) => self.computed_member(object, key, state)?,
            _ => self.compound_value(expr, state)?,
        };
        let mut values = values;
        self.cap(&mut values);
        Some(values)
    }
    fn identifier_value(&mut self, name: &str, state: State) -> Option<Vec<(State, Value)>> {
        if let Some(value) = state.env.get(name).cloned() {
            return Some(vec![(state, value)]);
        }
        let id = name.strip_prefix("binding:")?.parse().ok()?;
        if self.catalog.sink(self.file()?, id) {
            Some(vec![(state, Value::Executor)])
        } else {
            self.local_value(id, state)
        }
    }
    fn compound_value(&mut self, expr: &Expr, state: State) -> Option<Vec<(State, Value)>> {
        match expr {
            Expr::Object(entries) => self.object_values(entries, state),
            Expr::Array(items) => self.array_values(items, state),
            Expr::Call(name, args) => self.call(name, args, state),
            Expr::Template(pieces, holes) => self.template(pieces, holes, state, false),
            Expr::Sql(pieces, holes, true) => self.template(pieces, holes, state, true),
            Expr::Append(target, argument, _) => self.append(target, argument, state),
            Expr::Cond(test, yes, no) => self.conditional(test, yes, no, state),
            _ => self.operator(expr, state),
        }
    }
    pub(super) fn all(
        &mut self,
        expressions: &[&Expr],
        state: State,
    ) -> Option<Vec<(State, Vec<Value>)>> {
        let mut states = vec![(state, Vec::new())];
        for expression in expressions {
            let mut next = Vec::new();
            for (state, values) in states {
                for (state, value) in self.expression(expression, state)? {
                    if self
                        .sql_bytes(values.iter().map(Value::bytes).sum())
                        .is_none()
                    {
                        self.failures += 1;
                        continue;
                    }
                    let mut values = values.clone();
                    values.push(value);
                    next.push((state, values));
                }
                self.cap(&mut next);
            }
            states = next;
        }
        Some(states)
    }
}

include!("expressions/aggregates.rs");
