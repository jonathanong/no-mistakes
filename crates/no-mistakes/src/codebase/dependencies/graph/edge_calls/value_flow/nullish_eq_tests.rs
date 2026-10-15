use super::*;

fn dag(depth: usize, name: &str) -> NullishBranches {
    let mut value = Arc::new(FlowValue::Target(ResolvedCallTarget::Global {
        name: name.into(),
    }));
    for _ in 0..depth {
        value = Arc::new(FlowValue::Nullish(NullishBranches {
            value: Arc::clone(&value),
            fallback: value,
        }));
    }
    NullishBranches {
        value: Arc::clone(&value),
        fallback: value,
    }
}
#[test]
fn nullish_equality_visits_shared_pairs_once() {
    for depth in [2, 4, 8, 16, 32] {
        let left = dag(depth, "same");
        let right = dag(depth, "same");
        let (equal, visits) = left.compare(&right);
        assert!(equal);
        assert!(visits <= 2 * depth + 4, "depth={depth} visits={visits}");
        assert!(!left.compare(&dag(depth, "different")).0);
        assert!(left == right);
    }
}

fn branch(value: FlowValue) -> NullishBranches {
    let value = Arc::new(value);
    NullishBranches {
        value: Arc::clone(&value),
        fallback: value,
    }
}
fn typed(name: &str) -> FlowValue {
    FlowValue::Provenance {
        executor: Box::new(FlowValue::Nullish(dag(8, "target"))),
        typed: vec![("module".into(), name.into())],
        members: Vec::new(),
        receiver_factories: Vec::new(),
    }
}
#[test]
fn nullish_equality_preserves_receipts_and_wrapper_identity() {
    let value = FlowValue::Nullish(dag(8, "target"));
    let annotation = FlowValue::Annotated {
        executor: Box::new(value.clone()),
        file: "owner.mts".into(),
        source: Arc::new(
            crate::codebase::dependencies::extract::ValueTypeSource::Reference("Executor".into()),
        ),
    };
    let provenance = typed("Executor");
    let bound = FlowValue::Bound {
        target: Box::new(value.clone()),
        arguments: Vec::new(),
    };
    let returned = FlowValue::Returned {
        factory: vec![("module".into(), "factory".into())],
        executor: Box::new(value.clone()),
        receiver: None,
    };
    for wrapped in [
        annotation.clone(),
        provenance.clone(),
        FlowValue::Promise(Box::new(value)),
        bound.clone(),
        returned.clone(),
    ] {
        let left = branch(wrapped.clone());
        assert!(left == branch(wrapped));
        let shared = left.clone();
        assert!(left == shared);
    }
    let mut different = annotation.clone();
    if let FlowValue::Annotated { file, .. } = &mut different {
        *file = "other.mts".into();
    }
    assert!(branch(annotation.clone()) != branch(different));
    let mut different = annotation.clone();
    if let FlowValue::Annotated { source, .. } = &mut different {
        *source = Arc::new(
            crate::codebase::dependencies::extract::ValueTypeSource::Reference("Other".into()),
        );
    }
    assert!(branch(annotation) != branch(different));
    assert!(branch(provenance.clone()) != branch(typed("Other")));
    for member in [false, true] {
        let mut different = provenance.clone();
        if let FlowValue::Provenance {
            members,
            receiver_factories,
            ..
        } = &mut different
        {
            if member {
                members.push(("module".into(), "factory".into(), "query".into()));
            } else {
                receiver_factories.push(("module".into(), "factory".into()));
            }
        }
        assert!(branch(provenance.clone()) != branch(different));
    }
    let mut different = bound.clone();
    if let FlowValue::Bound { arguments, .. } = &mut different {
        arguments.push(BoundArguments {
            file: "owner.mts".into(),
            arguments: Arc::new(vec![ValueExpression::Undefined]),
            spread_at: None,
        });
    }
    assert!(branch(bound) != branch(different));
    for receiver in [false, true] {
        let mut different = returned.clone();
        if let FlowValue::Returned {
            factory,
            receiver: target,
            ..
        } = &mut different
        {
            if receiver {
                *target = Some(ResolvedCallTarget::Global {
                    name: "other".into(),
                });
            } else {
                factory.push(("other".into(), "factory".into()));
            }
        }
        assert!(branch(returned.clone()) != branch(different));
    }
}
