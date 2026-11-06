use super::*;

#[derive(Default)]
struct Observed {
    visits: usize,
    limit: usize,
    writes: Vec<ReceiverWrite>,
}
impl Extend<ReceiverWrite> for Observed {
    fn extend<I: IntoIterator<Item = ReceiverWrite>>(&mut self, values: I) {
        self.writes.extend(values);
    }
}
impl ReceiverOutput<RootRead> for Observed {
    fn visit(&mut self) -> bool {
        self.visits += 1;
        self.visits <= self.limit
    }
    fn active(&self) -> bool {
        self.visits <= self.limit
    }
}

#[test]
fn named_projection_cancels_before_constructing_receiver_coordinates() {
    let mut output = Observed::default();
    write_named_receiver_references::<RootRead>(
        "source.member",
        Some(0),
        (1, None),
        &[],
        &[],
        &mut output,
    );
    assert_eq!(output.visits, 1);
    assert!(output.writes.is_empty());
}
#[test]
fn wildcard_projection_stops_visiting_scalar_and_named_siblings_on_exhaustion() {
    for leaf in [
        ValueExpression::Unknown,
        ValueExpression::Named {
            name: "source".into(),
            scope: Some(0),
            offset: 1,
            context: None,
        },
    ] {
        let fields = (0..1024).map(|i| (i.to_string(), leaf.clone())).collect();
        let value = ValueExpression::Object(std::sync::Arc::new(fields), false);
        let mut output = Observed {
            limit: 7,
            ..Default::default()
        };
        write_receiver_references(&value, &[None], &[None], &mut output);
        assert_eq!(output.visits, 8, "no sibling visits after cancellation");
        assert_eq!(
            output.writes.len(),
            if matches!(leaf, ValueExpression::Unknown) {
                0
            } else {
                6
            }
        );
    }
}

#[test]
fn output_exhaustion_stops_either_fallback_before_constructing_another_receiver() {
    struct StopAfterOne(usize);
    impl Extend<ReceiverWrite> for StopAfterOne {
        fn extend<I: IntoIterator<Item = ReceiverWrite>>(&mut self, values: I) {
            self.0 += values.into_iter().count();
        }
    }
    impl ReceiverOutput<RootRead> for StopAfterOne {
        fn active(&self) -> bool {
            self.0 == 0
        }
        fn visit(&mut self) -> bool {
            assert_eq!(self.0, 0, "cancelled traversal visited a later expression");
            true
        }
    }
    let leaf = ValueExpression::Named {
        name: "source".into(),
        scope: Some(0),
        offset: 1,
        context: None,
    };
    let value =
        ValueExpression::Either(std::sync::Arc::new(leaf.clone()), std::sync::Arc::new(leaf));
    let mut output = StopAfterOne(0);
    write_receiver_references(&value, &[], &[], &mut output);
    assert_eq!(output.0, 1);
}
