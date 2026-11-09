use super::*;
pub(super) fn arguments(select: &Select) -> Vec<Expr> {
    struct Counts {
        depth: usize,
        out: Vec<Expr>,
    }
    impl Visitor for Counts {
        type Break = ();
        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth += 1;
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if self.depth != 0 {
                return ControlFlow::Continue(());
            }
            if let Expr::Function(function) = bare(expr) {
                if function.name.to_string().eq_ignore_ascii_case("count")
                    && function.over.is_none()
                    && function.within_group.is_empty()
                {
                    if let FunctionArguments::List(args) = &function.args {
                        if args.duplicate_treatment == Some(DuplicateTreatment::Distinct)
                            && (args.clauses.is_empty()
                                || super::super::expressions::count_ordering_matches(
                                    function, args,
                                ))
                        {
                            if let [FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))] =
                                args.args.as_slice()
                            {
                                self.out.push(expr.clone());
                            }
                        }
                    }
                }
            }
            ControlFlow::Continue(())
        }
    }
    let mut counts = Counts {
        depth: 0,
        out: Vec::new(),
    };
    if let Some(exprs) = projection(select) {
        for expr in exprs {
            let _ = expr.visit(&mut counts);
        }
    }
    let _ = select.having.visit(&mut counts);
    counts.out
}
