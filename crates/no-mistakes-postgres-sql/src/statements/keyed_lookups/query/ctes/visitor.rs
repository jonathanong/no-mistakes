use super::super::super::conditions::collect_pruned_children;
use super::{FxHashSet, QueryUses, RelationUses};
use crate::idents::ident_key;
use sqlparser::ast::{Expr, Ident, ObjectName, Query, SetExpr, TableFactor, Visitor};
use std::ops::ControlFlow;

fn insert_name(names: &mut FxHashSet<String>, name: Option<&Ident>) {
    if let Some(name) = name {
        names.insert(ident_key(name));
    }
}

fn inspect_table_commands(body: &SetExpr, names: &mut FxHashSet<String>) {
    match body {
        SetExpr::Table(table) if table.schema_name.is_none() => {
            insert_name(names, table.table_name.as_ref());
        }
        SetExpr::SetOperation { left, right, .. } => {
            inspect_table_commands(left, names);
            inspect_table_commands(right, names);
        }
        _ => {}
    }
}

impl Visitor for RelationUses {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.blocked_depth > 0 || self.blocked_roots.remove(&std::ptr::from_ref(expr)) {
            self.blocked_depth += 1;
        } else {
            collect_pruned_children(expr, &mut self.blocked_roots);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
        self.blocked_depth = self.blocked_depth.saturating_sub(1);
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.blocked_depth > 0 {
            return ControlFlow::Continue(());
        }
        let parent_definition = self
            .frames
            .last()
            .and_then(|parent| parent.named_definition(query));
        let mut current = QueryUses::new(query, parent_definition);
        inspect_table_commands(&query.body, &mut current.names);
        self.frames.push(current);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        if self.blocked_depth > 0 {
            return ControlFlow::Continue(());
        }
        let mut current = self.frames.pop().expect("query visit stack");
        let reachable = current.resolve();
        if let Some(parent) = self.frames.last_mut() {
            match current.parent_definition {
                Some(index) => parent.definitions[index].names = current.names,
                None => parent.names.extend(current.names),
            }
        } else {
            self.root_reachable = reachable;
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if self.blocked_depth > 0 {
            return ControlFlow::Continue(());
        }
        if let TableFactor::Table {
            name: ObjectName(parts),
            args: None,
            ..
        } = factor
        {
            if let [name] = parts.as_slice() {
                let current = self.frames.last_mut().expect("relation in query visit");
                insert_name(&mut current.names, name.as_ident());
            }
        }
        ControlFlow::Continue(())
    }
}
