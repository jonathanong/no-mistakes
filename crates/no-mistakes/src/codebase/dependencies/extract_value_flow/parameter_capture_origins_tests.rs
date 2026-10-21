use super::*;

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: Some(1),
        offset: 0,
        context: Some(CallableId(1)),
    }
}

#[test]
fn capture_generations_use_indexed_completion_predecessors() {
    let key = (1, "box".to_string());
    let mut origins = CaptureOrigins::default();
    origins.bindings.insert(
        key.clone(),
        Captures {
            initial: vec![named("initial")],
            owner: CallableId(1),
            foreign: false,
            events: (1..=96)
                .map(|generation| Reassignment {
                    key: key.clone(),
                    value: named(&format!("source{generation}")),
                    completed_at: 2 * generation,
                    context: Some(CallableId(1)),
                    certain: true,
                    recovered: false,
                })
                .collect(),
        },
    );
    for at in 0..=192 {
        let (values, possible) = origins.at(&key, (at, Some(CallableId(1)))).unwrap();
        assert!(!possible);
        let expected = if at < 2 {
            "initial".into()
        } else {
            format!("source{}", at / 2)
        };
        assert!(matches!(&values[0], ValueExpression::Named { name, .. } if name == &expected));
    }
    // Each actual partition comparison is counted, rather than scanning every
    // completed assignment again for every distinct capture coordinate.
    assert!(
        origins
            .comparisons
            .load(std::sync::atomic::Ordering::Relaxed)
            <= 8 * 193,
        "{} comparisons",
        origins
            .comparisons
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    assert!(origins.at(&key, (192, Some(CallableId(2)))).is_none());
    origins
        .bindings
        .get_mut(&key)
        .unwrap()
        .events
        .last_mut()
        .unwrap()
        .recovered = true;
    assert!(origins.at(&key, (192, Some(CallableId(1)))).unwrap().1);
    let captures = origins.bindings.get_mut(&key).unwrap();
    captures.events.last_mut().unwrap().certain = false;
    assert!(origins.at(&key, (192, Some(CallableId(1)))).is_none());
    let captures = origins.bindings.get_mut(&key).unwrap();
    captures.foreign = true;
    assert!(origins.at(&key, (0, Some(CallableId(1)))).is_none());
}

#[test]
fn missing_initial_origins_retain_later_forwarding_fallback() {
    let key = (1, "box".to_string());
    let mut origins = CaptureOrigins::default();
    origins.bindings.insert(
        key.clone(),
        Captures {
            initial: Vec::new(),
            owner: CallableId(1),
            foreign: false,
            events: vec![Reassignment {
                key: key.clone(),
                value: named("replacement"),
                completed_at: 10,
                context: Some(CallableId(1)),
                certain: true,
                recovered: false,
            }],
        },
    );
    assert!(origins.at(&key, (9, Some(CallableId(1)))).is_none());
    let (values, possible) = origins.at(&key, (10, Some(CallableId(1)))).unwrap();
    assert!(!possible);
    assert!(matches!(&values[0], ValueExpression::Named { name, .. } if name == "replacement"));
}
