use super::*;
use crate::codebase::postgres::embedded::helpers::program::{Mark, Program};
use std::sync::Arc;

fn lowered(expr: Expr) -> Lowered {
    Lowered {
        program: Arc::new(Program {
            params: Vec::new(),
            body: vec![Stmt::Return(Some(expr), Mark { line: 1, origin: 0 })],
        }),
        asynchronous: false,
        tags: Default::default(),
        references: Vec::new(),
    }
}

fn template(expr: Expr) -> Expr {
    Expr::Template(Vec::new(), vec![(expr, Mark { line: 1, origin: 0 })])
}

#[test]
fn closed_map_template_reads_keep_computed_keys_separate_from_bind_reads() {
    let slot = "binding:1";
    let map = || Expr::Ident(slot.into());
    let property = || Expr::Member(Box::new(map()), "sql".into());
    let computed =
        || Expr::ComputedMember(Box::new(map()), Box::new(Expr::Ident("binding:2".into())));
    assert!(use_shape(&lowered(map()), slot));
    assert!(use_shape(&lowered(property()), slot));
    assert!(use_shape(&lowered(computed()), slot));
    assert!(use_shape(&lowered(template(computed())), slot));
    assert!(bind_reads(&lowered(template(property())), slot));
    assert!(!bind_reads(&lowered(template(computed())), slot));
    assert!(!use_shape(&lowered(template(map())), slot));
    assert!(!use_shape(
        &lowered(template(Expr::ComputedMember(
            Box::new(map()),
            Box::new(map())
        ))),
        slot,
    ));
}

#[test]
fn nested_template_bind_containers_accept_scalars_without_exposing_the_map() {
    let values = vec![Expr::Bool(true), Expr::Null, Expr::Undefined, Expr::Num];
    let safe = template(Expr::Array(values));
    assert!(bind_reads(&lowered(template(safe)), "binding:1"));
    for expr in [
        Expr::Array(vec![Expr::Ident("binding:1".into())]),
        Expr::Call("unknown".into(), Vec::new()),
        Expr::Member(Box::new(Expr::Ident("binding:2".into())), "sql".into()),
        Expr::Opaque,
    ] {
        assert!(!bind_reads(&lowered(template(expr)), "binding:1"));
    }
    let mut opaque = lowered(Expr::Opaque);
    Arc::make_mut(&mut opaque.program).body = vec![Stmt::Opaque];
    assert!(!use_shape(&opaque, "binding:1"));
    assert!(!use_shape(&lowered(Expr::Num), "binding:1"));
}
