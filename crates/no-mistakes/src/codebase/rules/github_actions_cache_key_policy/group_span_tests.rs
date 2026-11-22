use super::super::super::{Event, Write};
use super::super::emit::{self, Pending};
use super::{after_first, commit};

#[test]
fn a_passed_write_is_not_emitted() {
    let mut events = Vec::new();
    commit(
        &mut events,
        vec![Pending::Pass(Write {
            name: "digest".to_string(),
            value: "$digest".to_string(),
            expand: true,
        })],
    );
    assert!(events.is_empty());
    commit(
        &mut events,
        vec![Pending::Event(Event::Write(Write {
            name: "digest".to_string(),
            value: "$digest".to_string(),
            expand: true,
        }))],
    );
    assert!(matches!(events.first(), Some(Event::Write(_))));
    assert_eq!(after_first(""), "");
    assert_eq!(after_first("( echo"), "echo");
    let mut outputs = crate::fx::fx_set();
    let mut sink = Vec::new();
    emit::record_closed(&mut sink, &mut outputs, "echo hi", false);
    assert!(sink.is_empty());
}
