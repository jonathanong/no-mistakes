use super::{collect_expr_names, collect_steps_names};
use crate::codebase::postgres::query_annotation::{Expr, Function, Step};
use crate::fx::{FxHashMap, FxHashSet};
use std::sync::Arc;

#[test]
fn collect_names_covers_every_step_expr_shadowing_and_early_exit() {
    let imports: FxHashMap<String, usize> = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "p", "q",
    ]
    .into_iter()
    .enumerate()
    .map(|(idx, name)| (name.to_string(), idx))
    .collect();

    let func = Arc::new(Function {
        start: 0,
        params: vec!["p".into()],
        self_name: Some("q".into()),
        body: vec![
            Step::Reserve(vec!["r".into()]),
            Step::Var("s".into()),
            Step::Hoisted("t".into(), Expr::Name("a".into())),
            Step::Bind("u".into(), Expr::Name("p".into())),
            Step::Branch(
                Expr::Name("q".into()),
                vec![Step::Return(Expr::Name("b".into()))],
                vec![Step::Effect(Expr::Name("r".into()))],
            ),
            Step::Append("x".into(), Expr::Name("c".into())),
            Step::Unsupported,
            Step::PotentialCalls(Vec::new()),
        ],
        supported: true,
        asynchronous: false,
        arrow: false,
        append_callback: None,
    });

    let steps = vec![
        Step::Reserve(vec!["local".into()]),
        Step::Var("v".into()),
        Step::Unsupported,
        Step::PotentialCalls(Vec::new()),
        Step::Branch(
            Expr::Await(Box::new(Expr::Name("d".into()))),
            vec![Step::Effect(Expr::ComputedMember(
                Box::new(Expr::Name("e".into())),
                Box::new(Expr::Name("f".into())),
            ))],
            vec![Step::Return(Expr::Call {
                callee: Box::new(Expr::Name("g".into())),
                args: vec![Expr::Name("h".into())],
                start: 0,
                end: 1,
            })],
        ),
        Step::Effect(Expr::Tagged(
            "i".into(),
            vec![Expr::Name("j".into())],
            vec![Expr::Name("k".into())],
        )),
        Step::Effect(Expr::OptionalCall {
            call: Box::new(Expr::Name("l".into())),
            children: vec![Expr::Name("m".into())],
        }),
        Step::Effect(Expr::SlotWrite {
            receiver: Box::new(Expr::Name("n".into())),
            index: 0,
            value: Box::new(Expr::Template(vec![Expr::Number(1), Expr::Unknown])),
        }),
        Step::Effect(Expr::Flow(vec![Step::Effect(Expr::Function(func))])),
    ];

    let mut names = FxHashSet::default();
    collect_steps_names(&steps, &imports, &mut names);
    for expected in [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ] {
        assert!(names.contains(expected), "missing {expected}");
    }
    assert!(!names.contains("p"));
    assert!(!names.contains("q"));

    // Early exit when all imports are already collected.
    let small: FxHashMap<String, usize> = [("a".into(), 0)].into_iter().collect();
    let mut full = FxHashSet::default();
    let binary = Expr::Binary(
        "+".into(),
        Box::new(Expr::Name("a".into())),
        Box::new(Expr::Name("a".into())),
    );
    collect_expr_names(&binary, &small, &mut full);
    assert_eq!(full.len(), 1);
    collect_steps_names(&steps, &small, &mut full);
    assert_eq!(full.len(), 1);
}
