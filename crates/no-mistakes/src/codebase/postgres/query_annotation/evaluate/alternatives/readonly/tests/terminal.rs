use super::references::{assert_arms, assert_read, assert_rejected};
use super::*;

pub(super) fn check_terminal<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &mut Evaluator<'_, F>,
    path: &Path,
    root: Environment,
    precise: bool,
) {
    let stateful = vec![
        Expr::Call {
            callee: Box::new(Expr::Name("chosen".into())),
            args: vec![],
            start: 0,
            end: 0,
        },
        Expr::ModuleLoad("./must-not-load.mts".into()),
        Expr::SlotWrite {
            receiver: Box::new(Expr::Name("slots".into())),
            index: 0,
            value: Box::new(Expr::Text("must-not-write".into())),
        },
    ];
    let failure = Value::Failure("evaluation-depth-limit");
    let expected = if precise {
        failure.clone()
    } else {
        Value::Aggregate(vec![failure.clone(), failure.clone(), failure].into())
    };
    let observer = InvocationObserver::new(true);
    {
        let _observer = crate::diagnostics::InvocationGuard::install(observer.clone());
        assert_arms(evaluator, path, root, &stateful, expected, 0);
    }
    assert_eq!(
        observer.snapshot().work["postgres.annotation.depth_limited_alternatives"],
        1
    );
    // The top-level exception must not weaken recursive namespace checks.
    let namespace = Expr::Member(Box::new(Expr::Name("unloaded".into())), "field".into());
    assert!(evaluator
        .readonly_alternatives(&[namespace], path, &root, 1, true)
        .is_none());

    let left = evaluator.scopes[root]["left"].clone();
    let right = evaluator.scopes[root]["right"].clone();
    let records = vec![
        Value::Record(vec![left.clone()].into(), vec!["exact".into()]),
        Value::Record(vec![right].into(), vec!["exact".into()]),
    ]
    .into();
    let expected = if precise {
        Value::Joined(records)
    } else {
        Value::Aggregate(records)
    };
    assert!(evaluator.scopes[root]["chosenRecord"] == expected);
    let records = vec![
        Value::Record(
            vec![if precise {
                Value::Boolean(true)
            } else {
                Value::Unknown
            }]
            .into(),
            vec!["enabled".into()],
        ),
        Value::Record(vec![].into(), vec![]),
    ]
    .into();
    let expected = if precise {
        Value::Joined(records)
    } else {
        Value::Aggregate(records)
    };
    assert!(evaluator.scopes[root]["chosenEmpty"] == expected);
    for (expression, expected) in [
        (
            Expr::Record(vec![], vec![]),
            Value::Record(vec![].into(), vec![]),
        ),
        (Expr::Object(vec![]), Value::Object(vec![].into())),
        (Expr::Container(vec![]), Value::Aggregate(vec![].into())),
        (Expr::Children(vec![]), Value::References(vec![].into())),
        (
            Expr::Record(vec![Expr::Name("left".into())], vec!["exact".into()]),
            Value::Record(vec![left.clone()].into(), vec!["exact".into()]),
        ),
        (
            Expr::Object(vec![Expr::Name("left".into())]),
            Value::Object(vec![left.clone()].into()),
        ),
        (
            Expr::Container(vec![Expr::Name("left".into())]),
            Value::Aggregate(vec![left.clone()].into()),
        ),
        (
            Expr::Children(vec![Expr::Name("left".into())]),
            Value::References(vec![left.clone()].into()),
        ),
        (
            Expr::Record(
                vec![Expr::Object(vec![Expr::Name("left".into())])],
                vec!["nested".into()],
            ),
            Value::Record(
                vec![Value::Object(vec![left.clone()].into())].into(),
                vec!["nested".into()],
            ),
        ),
    ] {
        assert_read(evaluator, path, root, expression, expected, precise);
    }
    if precise {
        assert_read(
            evaluator,
            path,
            root,
            Expr::Record(
                vec![Expr::Member(
                    Box::new(Expr::Name("record".into())),
                    "exact".into(),
                )],
                vec!["exact".into()],
            ),
            Value::Record(vec![left].into(), vec!["exact".into()]),
            precise,
        );
    }
    evaluator.scopes[root].insert("accessor".into(), Value::Unknown);
    let accessor = Expr::Member(Box::new(Expr::Name("accessor".into())), "field".into());
    for constructor in [
        Expr::Record(vec![accessor.clone()], vec!["field".into()]),
        Expr::Object(vec![accessor.clone()]),
        Expr::Container(vec![accessor.clone()]),
        Expr::Children(vec![accessor]),
        Expr::Array(vec![]),
        Expr::Sequence(vec![]),
        Expr::Template(vec![]),
        Expr::Opaque(vec![]),
        evaluator.files[path].facts.globals["callback"].clone(),
    ] {
        assert_rejected(evaluator, path, root, &[constructor]);
    }
    for stateful in stateful {
        let mixed = [Expr::Object(vec![]), stateful];
        assert_rejected(evaluator, path, root, &mixed);
    }
}
