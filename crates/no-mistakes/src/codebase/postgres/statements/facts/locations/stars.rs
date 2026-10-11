use super::{at, Star};
use crate::codebase::postgres::idents::object_name_key;
use crate::codebase::postgres::statements::select::arg_expr;
use sqlparser::ast::{
    Expr, FunctionArgExpr, FunctionArguments, SelectItem, SelectItemQualifiedWildcardKind, Spanned,
    Visit, Visitor,
};
use std::ops::ControlFlow;

pub(super) fn items(items: &[SelectItem]) -> Vec<Star> {
    let mut out = Vec::new();
    for item in items {
        match item {
            SelectItem::Wildcard(_) => out.push(Star {
                qualifier: None,
                function: None,
                at: at(item.span()),
            }),
            SelectItem::QualifiedWildcard(kind, _) => {
                if let SelectItemQualifiedWildcardKind::ObjectName(name) = kind {
                    out.push(Star {
                        qualifier: Some(object_name_key(name)),
                        function: None,
                        at: at(item.span()),
                    });
                }
            }
            _ => {
                let _: ControlFlow<()> = item.visit(&mut Functions {
                    out: &mut out,
                    depth: 0,
                });
            }
        }
    }
    out
}
struct Functions<'a> {
    out: &'a mut Vec<Star>,
    depth: usize,
}
impl Visitor for Functions<'_> {
    type Break = ();
    fn pre_visit_query(&mut self, _: &sqlparser::ast::Query) -> ControlFlow<()> {
        self.depth += 1;
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> ControlFlow<()> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.depth == 0 {
            if let Expr::Function(function) = expr {
                if let FunctionArguments::List(list) = &function.args {
                    for arg in &list.args {
                        if let FunctionArgExpr::QualifiedWildcard(name) = arg_expr(arg) {
                            self.out.push(Star {
                                qualifier: Some(object_name_key(name)),
                                function: Some(
                                    crate::codebase::postgres::schema::relation_name(
                                        &function.name,
                                    )
                                    .to_ascii_lowercase(),
                                ),
                                at: at(name.span()),
                            });
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }
}
