use super::read;

#[test]
fn captured_wrappers_reject_different_callables_receivers_and_rewritten_arguments() {
    let read = read("captured-wrapper-rejections.mts");
    assert!(read.provenance().pass_through.is_empty());
    assert!(read.facts.transparent_receiver_writes.is_empty());
}

#[test]
fn captured_wrappers_allow_direct_truthiness_inspection_and_fs_extra_identity() {
    for name in ["captured-wrapper-if.mjs", "captured-wrapper-extra.mjs"] {
        let read = read(name);
        assert_eq!(read.provenance().pass_through.len(), 1, "{name}");
        assert_eq!(read.facts.transparent_receiver_writes.len(), 1, "{name}");
    }
    let spy = read("captured-wrapper-spy.mjs");
    assert_eq!(spy.provenance().pass_through.len(), 1);
    assert!(spy.facts.transparent_receiver_writes.is_empty());
}

#[test]
fn captured_wrappers_keep_unproven_restorations_escapes_and_excessive_receiver_uses() {
    for suffix in [
        "restore-operator",
        "restore-member",
        "restore-mutable",
        "restore-late",
        "restore-destructured",
        "restore-nested",
        "method-escape",
        "receiver-limit",
    ] {
        let name = format!("captured-wrapper-{suffix}.mjs");
        let read = read(&name);
        assert_eq!(read.provenance().pass_through.len(), 1, "{name}");
        assert!(read.facts.transparent_receiver_writes.is_empty(), "{name}");
    }
}
