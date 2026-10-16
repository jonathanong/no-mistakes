use super::*;

#[test]
fn compact_leaf_does_not_grow_the_origin_arena() {
    // This is the exact old arena shape, retained only as a layout oracle.
    enum LegacyOrigin {
        Value(ValueExpression),
        Either(usize, usize),
    }
    for value in [
        LegacyOrigin::Value(ValueExpression::Unknown),
        LegacyOrigin::Either(0, 0),
    ] {
        match value {
            LegacyOrigin::Value(value) => {
                std::hint::black_box(value);
            }
            LegacyOrigin::Either(left, right) => {
                std::hint::black_box((left, right));
            }
        }
    }
    use std::mem::size_of;
    println!(
        "COMPACT_LAYOUT origin={} legacy={} value={} single={} names={} optional_names={}",
        size_of::<Origin>(),
        size_of::<LegacyOrigin>(),
        size_of::<ValueExpression>(),
        size_of::<SingleNamedField>(),
        size_of::<Names>(),
        size_of::<Option<Box<Names>>>()
    );
    assert_eq!(size_of::<Origin>(), size_of::<LegacyOrigin>());
    assert_eq!(size_of::<Option<Box<Names>>>(), size_of::<usize>());
}

fn projection_parity<R: nested::RootCoordinate + std::fmt::Debug + PartialEq>() {
    for scope in [None, Some(0), Some(7)] {
        for name in ["client", "client.nested"] {
            let read = (31, Some(CallableId(3)));
            let mut origins = ReceiverBindingOrigins::default();
            let compact = origins.single_named_field("field", name, scope, read);
            let full = ValueExpression::Object(
                Arc::new(FxHashMap::from_iter([(
                    "field".to_string(),
                    ValueExpression::Named {
                        name: name.to_string(),
                        scope,
                        offset: read.0,
                        context: read.1,
                    },
                )])),
                false,
            );
            for path in [
                vec![],
                vec![Some("other".into())],
                vec![None],
                vec![Some("field".into()), Some("get".into())],
                vec![None, Some("get".into())],
            ] {
                let reads = vec![Some((81, None)); path.len()];
                let mut canonical: Vec<nested::ReceiverWrite<R>> = Vec::new();
                let mut projected: Vec<nested::ReceiverWrite<R>> = Vec::new();
                nested::write_receiver_references(&full, &path, &reads, &mut canonical);
                origins.append_single(&compact, &path, &reads, &mut projected);
                assert_eq!(
                    canonical, projected,
                    "scope={scope:?} name={name} path={path:?}"
                );
            }
        }
    }
}
#[test]
fn compact_projection_preserves_both_root_coordinate_modes() {
    projection_parity::<()>();
    projection_parity::<nested::RootRead>();
}

#[test]
fn named_projection_preserves_default_and_awaited_alternatives() {
    let named = |name: &str| ValueExpression::Named {
        name: name.into(),
        scope: Some(4),
        offset: 17,
        context: None,
    };
    let value = ValueExpression::Default(
        Arc::new(named("client.nested")),
        Arc::new(ValueExpression::Awaited(Arc::new(named("fallback")))),
    );
    let path = vec![Some("get".into())];
    let mut writes: Vec<nested::ReceiverWrite> = Vec::new();
    nested::write_receiver_references(&value, &path, &[Some((31, None))], &mut writes);
    assert_eq!(writes.len(), 2);
    assert_eq!(
        writes[0].0,
        (
            (4, "client".into()),
            vec![Some("nested".into()), Some("get".into())]
        )
    );
    assert_eq!(writes[0].1, vec![Some((17, None)), Some((31, None))]);
    assert_eq!(writes[1].0, ((4, "fallback".into()), path));
    assert_eq!(writes[1].1, vec![Some((31, None))]);
    assert_eq!(writes[0].2, (17, None));
}

#[path = "mutable_origin_certificate_tests.rs"]
mod certificates;
