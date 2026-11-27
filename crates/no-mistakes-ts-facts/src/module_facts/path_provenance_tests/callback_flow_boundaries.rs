use super::*;

#[test]
fn owned_callback_paths_require_invocation_arguments_and_completion_ownership() {
    let read = read("3304-callback-flow.mts");
    for (case, expected) in [
        ("sync-block", PathAnchor::Temp),
        ("unawaited-concise", PathAnchor::Unknown),
        ("unawaited-async", PathAnchor::Unknown),
        ("awaited-concise", PathAnchor::Temp),
        ("awaited-function", PathAnchor::Temp),
        ("missing-argument", PathAnchor::Unknown),
        ("missing-parameter", PathAnchor::Unknown),
        ("unused-parameter", PathAnchor::Unknown),
        ("arrow-helper", PathAnchor::Temp),
    ] {
        let origin = read.origin(&format!("readdirSync(root, '{case}')"));
        assert_eq!(origin.anchor, expected, "{case}: {origin:?}");
        if expected == PathAnchor::Temp {
            assert!(origin.param.is_none(), "{case}: {origin:?}");
            assert!(origin.segments.is_empty(), "{case}: {origin:?}");
        }
    }
}
