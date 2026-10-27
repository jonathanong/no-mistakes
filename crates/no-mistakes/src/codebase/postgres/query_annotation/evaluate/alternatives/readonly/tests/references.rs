use super::*;

pub(super) fn check_reads<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &mut Evaluator<'_, F>,
    path: &Path,
    root: Environment,
    precise: bool,
) {
    let left = evaluator.scopes[root]["left"].clone();
    let right = evaluator.scopes[root]["right"].clone();
    if precise {
        for (name, incoming, alternative) in [
            ("nullish", Value::Boolean(true), Value::Boolean(false)),
            ("logical", Value::Number(4), Value::Number(7)),
        ] {
            let skipped = Value::References(vec![].into());
            let expected = Value::Aggregate(
                vec![incoming, Value::Joined(vec![alternative, skipped].into())].into(),
            );
            assert!(
                evaluator.scopes[root][name] == expected,
                "{name} must retain concrete control values and its empty skipped arm"
            );
        }
        let mapped = 30_000;
        let args = evaluator.scopes[root]["args"].clone();
        evaluator.argument_objects.insert(mapped, vec![args]);
        evaluator
            .mapped_arguments
            .insert(root, vec![(mapped, vec!["args".into()])]);
        evaluator
            .mapped_parameter_indices
            .insert(mapped, FxHashMap::from_iter([("args".into(), 0)]));
        assert_read(
            evaluator,
            path,
            root,
            Expr::Member(Box::new(Expr::Name("args".into())), "field".into()),
            Value::Boolean(true),
            precise,
        );
        let read = Expr::Member(Box::new(Expr::Name("record".into())), "exact".into());
        assert_read(evaluator, path, root, read, left.clone(), precise);
        let nested = Expr::Member(
            Box::new(Expr::Member(
                Box::new(Expr::Name("record".into())),
                "nested".into(),
            )),
            "exact".into(),
        );
        assert_read(evaluator, path, root, nested, right, precise);
        assert_read(
            evaluator,
            path,
            root,
            Expr::Index(Box::new(Expr::Name("exactArray".into())), 0),
            left.clone(),
            precise,
        );
    }
    // Use an existing saved array in precise mode; supply its legacy model
    // explicitly because legacy source arrays use aggregate values instead.
    let id = 10_000;
    evaluator.argument_objects.insert(id, vec![left.clone()]);
    evaluator.scopes[root].insert("slots".into(), Value::Array(id));
    evaluator.definite_deleted_argument_slots.insert((id, 0));
    let index = Expr::Index(Box::new(Expr::Name("slots".into())), 0);
    assert_read(
        evaluator,
        path,
        root,
        index.clone(),
        Value::Unknown,
        precise,
    );
    evaluator.recreated_argument_slots.insert((id, 0));
    assert_read(evaluator, path, root, index.clone(), left.clone(), precise);
    evaluator.invalidated_builders.insert(id);
    assert_read(
        evaluator,
        path,
        root,
        index,
        Value::Possible(vec![left.clone()].into()),
        precise,
    );
    assert_read(
        evaluator,
        path,
        root,
        Expr::Name("slots".into()),
        Value::Array(id),
        precise,
    );
    let callback = evaluator.scopes[root]["callback"].clone();
    assert_read(
        evaluator,
        path,
        root,
        Expr::Name("callback".into()),
        callback.clone(),
        precise,
    );
    evaluator.scopes[root].insert("chosen".into(), callback);
    let call = Expr::Call {
        callee: Box::new(Expr::Name("chosen".into())),
        args: vec![],
        start: 0,
        end: 0,
    };
    assert!(evaluator.expr(&call, path, &root, 16, false).exposed() == left);

    let builder = 20_000;
    evaluator.scopes[root].insert(
        "invalid".into(),
        Value::Prefix("/* invalid */ SELECT 4".into(), true, Some(builder)),
    );
    evaluator.invalidated_builders.insert(builder);
    assert_read(
        evaluator,
        path,
        root,
        Expr::Name("invalid".into()),
        if precise {
            Value::Failure("opaque-builder-effect")
        } else {
            Value::Unknown
        },
        precise,
    );

    // A dirty mapped parameter must read its argument candidates, rather than
    // trusting a stale scalar binding in the lexical environment.
    evaluator.scopes[root].insert("dirty".into(), Value::Unknown);
    evaluator
        .mapped_arguments
        .insert(root, vec![(id, vec!["dirty".into()])]);
    evaluator
        .mapped_parameter_indices
        .insert(id, FxHashMap::from_iter([("dirty".into(), 0)]));
    let dirty = Expr::Name("dirty".into());
    assert!(evaluator
        .readonly_alternatives(&[dirty], path, &root, 16, true)
        .is_none());
    let namespace = Expr::Member(Box::new(Expr::Name("unloaded".into())), "query".into());
    assert!(evaluator
        .readonly_alternatives(&[namespace], path, &root, 16, true)
        .is_none());
    let mut deep = Expr::Name("left".into());
    for _ in 0..32 {
        deep = Expr::Member(Box::new(deep), "field".into());
    }
    assert!(evaluator
        .readonly_alternatives(&[deep], path, &root, 16, true)
        .is_none());
}

fn assert_read<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &mut Evaluator<'_, F>,
    path: &Path,
    root: Environment,
    expression: Expr,
    expected: Value,
    precise: bool,
) {
    macro_rules! unchanged {
        ($($field:ident),+ $(,)?) => {{
            $(let $field = evaluator.$field.clone();)+
            let result = evaluator.readonly_alternatives(&[expression], path, &root, 16, true);
            let expected = crate::codebase::postgres::query_annotation::evaluate::concat::alternative_result(vec![expected], precise);
            assert!(result == Some(expected));
            $(assert!(evaluator.$field == $field, "{} changed during a read", stringify!($field));)+
        }};
    }
    unchanged!(
        scopes,
        modules,
        events,
        next_builder,
        argument_objects,
        argument_extra_slots,
        invalidated_builders,
        builder_updates,
        captured_bindings,
        captured_binding_readers,
        active_callback_functions,
        active_callback_executions,
        mapped_arguments,
        mapped_argument_owners,
        mapped_parameter_indices,
        deleted_argument_slots,
        definite_deleted_argument_slots,
        disconnected_argument_slots,
        recreated_argument_slots,
        untrusted_array_intrinsics,
        fresh_mapped_parameters,
        fresh_mapped_argument_bindings
    );
}
