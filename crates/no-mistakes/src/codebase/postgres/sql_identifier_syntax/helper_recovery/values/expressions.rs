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
            Expr::Ident(name) => {
                let value = state.env.get(name).cloned().or_else(|| {
                    let id = name.strip_prefix("binding:")?.parse().ok()?;
                    self.catalog
                        .sink(self.file()?, id)
                        .then_some(Value::Executor)
                })?;
                vec![(state, value)]
            }
            Expr::Member(object, name) => self.member(object, name, state)?,

            Expr::Object(entries) => {
                let expressions = entries
                    .iter()
                    .map(|entry| match entry {
                        Entry::Prop(_, value) => Some(value),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                self.all(&expressions, state)?
                    .into_iter()
                    .map(|(state, values)| {
                        let props = entries
                            .iter()
                            .zip(values)
                            .filter_map(|(entry, value)| match entry {
                                Entry::Prop(key, _) => Some((key.clone(), value)),
                                _ => None,
                            })
                            .collect();
                        (state, Value::Object(props))
                    })
                    .collect()
            }
            Expr::Array(items) => self
                .all(&items.iter().collect::<Vec<_>>(), state)?
                .into_iter()
                .map(|(state, items)| (state, Value::Array(items)))
                .collect(),
            Expr::Call(name, args) => self.call(name, args, state)?,
            Expr::Template(pieces, holes) => self.template(pieces, holes, state, false)?,
            Expr::Sql(pieces, holes, true) => self.template(pieces, holes, state, true)?,
            Expr::Append(target, argument, _) => self.append(target, argument, state)?,

            Expr::Cond(test, yes, no) => self.conditional(test, yes, no, state)?,

            _ => self.operator(expr, state)?,
        };
        let mut values = values;
        self.cap(&mut values);
        Some(values)
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
