use crate::codebase::postgres::statement_facts::SqlGroupingQuery;
use sqlparser::ast::{Query, SetExpr, Statement, Visit, Visitor};
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
    });
}

struct Roots<'a> {
    out: &'a mut Vec<SqlGroupingQuery>,
    depth: usize,
    parameter_sites: &'a Arc<[(u32, u32)]>,
}

impl Visitor for Roots<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.depth == 0 {
            super::locations::work::grouping_root();
            self.out.push(SqlGroupingQuery {
                query: Box::new(query.clone()),
                sites: Vec::new(),
                parameter_sites: Arc::clone(self.parameter_sites),
            });
        }
        self.depth += 1;
        select_sites(&query.body, &mut self.out.last_mut().unwrap().sites);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.depth -= 1;
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
