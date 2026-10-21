use super::*;

#[test]
fn only_selected_roots_prove_identity() {
    let missing = RootVersion {
        context: None,
        selected: None,
        generation: 0,
    };
    assert_eq!(missing.known_identity(), None);
    let selected = RootVersion {
        selected: Some(3),
        ..missing
    };
    assert_eq!(selected.known_identity(), Some(selected.identity()));
    // Arena generation is not object identity; callable context is.
    assert!(selected.same_selected(&RootVersion {
        generation: 7,
        ..selected
    }));
    assert!(!selected.same_selected(&RootVersion {
        context: Some(CallableId(1)),
        ..selected
    }));
}
