use super::super::super::File;
use super::{Task, Tasks};
use crate::codebase::{
    dependencies::extract::ExportedBinding,
    postgres::query_annotation::{modules::Module, Expr, Function, QueryAnnotationFileFacts},
    ts_source::facts::TsFileFacts,
};
use std::sync::Arc;

fn function(start: u32) -> Expr {
    Expr::Function(Arc::new(Function {
        start,
        params: Vec::new(),
        body: Vec::new(),
        supported: true,
        asynchronous: false,
        arrow: false,
        self_name: None,
    }))
}
fn call(start: u32) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Unknown),
        args: Vec::new(),
        start,
        end: start + 1,
    }
}

#[test]
fn task_order_preserves_filters_and_ordinary_private_functions() {
    for precise in [false, true] {
        let facts = QueryAnnotationFileFacts {
            precise_records: precise,
            globals: [
                ("a".into(), function(10)),
                ("b_private".into(), function(20)),
                (
                    "c_registry".into(),
                    Expr::Array(vec![function(30), function(10)]),
                ),
                ("d_duplicate_private".into(), function(10)),
            ]
            .into(),
            unmodeled_calls: vec![Expr::Unknown, call(99), call(100), call(101)],
            branch_calls: [(99, 100)].into_iter().collect(),
            ..Default::default()
        };
        let ts = TsFileFacts {
            exported_bindings: ["a", "c_registry"]
                .map(|name| ExportedBinding {
                    specifier: None,
                    local: name.into(),
                    exported: name.into(),
                })
                .into(),
            ..Default::default()
        };
        let file = File {
            facts: &facts,
            module: Arc::new(Module::new(ts.into())),
            executors: [(100, 101)].into_iter().collect(),
        };
        let actual: Vec<_> = Tasks::new(&file)
            .map(|task| match task {
                Task::Call(Expr::Call { start, .. }) => ("call", *start),
                Task::Function(function) => ("function", function.start),
                Task::Call(_) => panic!("only actual Call expressions are tasks"),
            })
            .collect();
        let expected = if precise {
            vec![("call", 101), ("function", 10), ("function", 30)]
        } else {
            vec![
                ("call", 99),
                ("call", 101),
                ("function", 10),
                ("function", 20),
                ("function", 10),
            ]
        };
        assert_eq!(actual, expected);
    }
}
