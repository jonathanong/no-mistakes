use crate::codebase::postgres::statement_facts::SqlGroupingQuery;
use sqlparser::ast::{Query, SetExpr, Statement, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlGroupingQuery>, enabled: bool) {
    if !enabled {
        return;
    }
    let _: ControlFlow<()> = statement.visit(&mut Roots { out, depth: 0 });
}

struct Roots<'a> {
    out: &'a mut Vec<SqlGroupingQuery>,
    depth: usize,
}

impl Visitor for Roots<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.depth == 0 {
            super::locations::work::grouping_root();
            self.out.push(SqlGroupingQuery {
                query: Box::new(query.clone()),
                sites: Vec::new(),
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
