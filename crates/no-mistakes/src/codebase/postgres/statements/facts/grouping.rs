use crate::codebase::postgres::statement_facts::SqlGroupingQuery;
use sqlparser::ast::{
    Expr, GroupByExpr, ObjectNamePart, Query, Select, SetExpr, Statement, Visit, Visitor,
};
use std::{ops::ControlFlow, sync::Arc};

pub(super) fn collect(
    statement: &Statement,
    out: &mut Vec<SqlGroupingQuery>,
    parameter_sites: Option<&Arc<[(u32, u32)]>>,
) {
    let Some(parameter_sites) = parameter_sites else {
        return;
    };
    let _: ControlFlow<()> = statement.visit(&mut Roots {
        out,
        depth: 0,
        parameter_sites,
        sites: Vec::new(),
        candidate: false,
    });
}

struct Roots<'a> {
    out: &'a mut Vec<SqlGroupingQuery>,
    depth: usize,
    parameter_sites: &'a Arc<[(u32, u32)]>,
    sites: Vec<(usize, usize)>,
    candidate: bool,
}

impl Visitor for Roots<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.depth == 0 {
            self.sites.clear();
            self.candidate = false;
        }
        self.depth += 1;
        select_sites(&query.body, &mut self.sites);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.depth -= 1;
        if self.depth == 0 && self.candidate {
            super::locations::work::grouping_root();
            self.out.push(SqlGroupingQuery {
                query: Arc::new(query.clone()),
                sites: std::mem::take(&mut self.sites),
                parameter_sites: Arc::clone(self.parameter_sites),
            });
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
        self.candidate |= select.distinct.is_some()
            || !matches!(&select.group_by, GroupByExpr::Expressions(expressions, modifiers)
                if expressions.is_empty() && modifiers.is_empty());
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let Expr::Function(function) = expr {
            self.candidate |= matches!(function.name.0.last(), Some(ObjectNamePart::Identifier(name))
                if name.value.eq_ignore_ascii_case("count"));
        }
        ControlFlow::Continue(())
    }
}

fn select_sites(body: &SetExpr, out: &mut Vec<(usize, usize)>) {
    match body {
        SetExpr::Select(select) => {
            let at = select.select_token.0.span.start;
            out.push((at.line as usize, at.column as usize));
        }
        SetExpr::SetOperation { left, right, .. } => {
            select_sites(left, out);
            select_sites(right, out);
        }
        _ => {}
    }
}

#[cfg(test)]
mod pruning_tests;
