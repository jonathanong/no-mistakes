use super::*;

fn callback_start(read: &Read, title: &str) -> u32 {
    let call = read.source.find(title).unwrap();
    (call + read.source[call..].find("() =>").unwrap()) as u32
}

#[test]
fn concurrency_options_preserve_known_timeouts_and_reject_unknown_options() {
    let read = read("coverage/concurrent-options.ts");
    let spans = &read.provenance().lifecycle_overlaps;
    assert!(spans.contains(&(0, u32::MAX)));
    for title in [
        "primitive options",
        "spread options",
        "computed options",
        "quoted runner",
    ] {
        assert!(
            spans
                .iter()
                .any(|(start, _)| *start == callback_start(&read, title)),
            "{title}"
        );
    }
    for title in ["numeric timeout", "known other option"] {
        assert!(
            !spans
                .iter()
                .any(|(start, _)| *start == callback_start(&read, title)),
            "{title}"
        );
    }
}

#[test]
fn concurrent_callbacks_follow_function_expressions_but_not_changed_or_opaque_values() {
    let read = read("coverage/concurrent-named.ts");
    let spans = &read.provenance().lifecycle_overlaps;
    let expression = read.source.find("function expressionBody").unwrap() as u32;
    assert!(spans.iter().any(|(start, _)| *start == expression));
    assert!(spans.iter().filter(|span| **span == (0, u32::MAX)).count() >= 4);
    let changed = read.source.find("() =>").unwrap() as u32;
    assert!(!spans.iter().any(|(start, _)| *start == changed));
    let typed = callback_start(&read, "type only");
    assert!(!spans.iter().any(|(start, _)| *start == typed));
}

#[test]
fn hook_bindings_reset_callbacks_and_getter_shapes_remain_conservative() {
    let read = read("coverage/hooks-getters.ts");
    let paths = read.provenance();
    assert!(matches!(
        write(paths, "QUOTED").scope,
        WriteScope::BeforeEach(..)
    ));
    assert!(matches!(
        write(paths, "TYPED").scope,
        WriteScope::Function(..)
    ));
    assert!(paths.env_unknown_write);
    let cleanup = read.source.find("function cleanup()").unwrap() as u32;
    assert!(paths
        .cleanup_callbacks
        .iter()
        .any(|(start, _)| *start == cleanup));
    assert_eq!(paths.env_resets.len(), 1);
    let valid = read.source.find("function validFallback()").unwrap() as u32;
    assert_eq!(paths.env_getters.len(), 1);
    assert_eq!(paths.env_getters[&valid].env.as_deref(), Some("X"));
}
