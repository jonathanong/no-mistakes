use super::*;
use sqlparser::ast::{Expr, GroupByExpr, Select, TableFactor, Visitor};

impl Visitor for Scan {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        let mut frame = Frame {
            previous_ctes: self.ctes.clone(),
            split_selects: matches!(&*query.body, SetExpr::SetOperation { .. }),
            enclosing: self
                .derived_scopes
                .remove(&(query as *const Query as usize)),
            ..Frame::default()
        };
        for name in output_names(query, self.positions.as_deref()) {
            *frame.labels.entry(name).or_default() += 1;
        }
        self.prepare_ctes(query);
        self.stack.push(frame);
        ControlFlow::Continue(())
    }

    /// The relations of a level are complete only after it has been visited (the projection comes
    /// before FROM), so its references are resolved here and what remains moves up a level.
    fn post_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.finish_frame(query)
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
        if self.stack.last().is_some_and(|frame| frame.split_selects) {
            let mut frame = Frame {
                select_frame: true,
                ..Frame::default()
            };
            // GROUP BY labels are local to a SELECT arm; the query's ORDER BY labels remain on
            // its parent frame because ORDER BY applies to the combined set-operation output.
            if let GroupByExpr::Expressions(expressions, _) = &select.group_by {
                let labels: BTreeSet<_> = select
                    .projection
                    .iter()
                    .filter_map(columns::label_name)
                    .collect();
                for expr in expressions {
                    if let Expr::Identifier(ident) = expr {
                        let name = ident_key(ident);
                        if labels.contains(&name) {
                            *frame.labels.entry(name).or_default() += 1;
                        }
                    }
                }
            }
            self.stack.push(frame);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_select(&mut self, _: &Select) -> ControlFlow<()> {
        if self.stack.last().is_some_and(|frame| frame.select_frame) {
            self.finish_frame_without_query()
        } else {
            ControlFlow::Continue(())
        }
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if let TableFactor::Derived {
            subquery, lateral, ..
        } = factor
        {
            let scope = if *lateral {
                self.stack
                    .last()
                    .map(|frame| frame.scope.clone())
                    .unwrap_or_default()
            } else {
                Scope::default()
            };
            self.derived_scopes
                .insert(&**subquery as *const Query as usize, scope);
        }
        if matches!(factor, TableFactor::NestedJoin { alias: Some(_), .. }) {
            if let Some(frame) = self.stack.last_mut() {
                self.join_scopes.push(JoinScope {
                    outside: frame.scope.begin_join(),
                    qualifiers: frame.qualifiers.len(),
                    escaping_qualifiers: frame.escaping_qualifiers.len(),
                    bare: frame.bare.clone(),
                });
            }
            return ControlFlow::Continue(());
        }
        self.add_factor(factor);
        ControlFlow::Continue(())
    }

    fn post_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if matches!(factor, TableFactor::NestedJoin { alias: Some(_), .. }) {
            if let (Some(frame), Some(scope)) = (self.stack.last_mut(), self.join_scopes.pop()) {
                // ON and LATERAL reads use the child namespace before the join alias hides it.
                let internal = frame.qualifiers.split_off(scope.qualifiers);
                frame.qualifiers.extend(
                    internal
                        .into_iter()
                        .filter(|read| !frame.scope.relations.contains(&read.key)),
                );
                let escaping = frame
                    .escaping_qualifiers
                    .split_off(scope.escaping_qualifiers);
                frame.escaping_qualifiers.extend(
                    escaping
                        .into_iter()
                        .filter(|read| !frame.scope.relations.contains(&read.key)),
                );
                frame.bare.retain(|name, count| {
                    if frame.scope.whole_rows.contains(name) {
                        *count = scope.bare.get(name).copied().unwrap_or(0);
                    }
                    *count > 0
                });
                frame.scope.finish_join(scope.outside);
                self.add_factor(factor);
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        let Some(frame) = self.stack.last_mut() else {
            return ControlFlow::Continue(());
        };
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
                frame.qualifiers.push(super::Qualified {
                    key: parts[..parts.len() - 1].iter().map(ident_key).collect(),
                    sql: parts[..parts.len() - 1]
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("."),
                    scopes: Vec::new(),
                });
            }
            Expr::Identifier(ident)
                if !is_placeholder_ident_at(ident, self.positions.as_deref()) =>
            {
                *frame.bare.entry(ident_key(ident)).or_default() += 1;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
