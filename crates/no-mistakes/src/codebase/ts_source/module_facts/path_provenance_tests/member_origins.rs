use super::*;

#[test]
fn immutable_owned_members_and_awaited_object_returns_are_paths() {
    let read = read("members.mts");
    for needle in [
        "readdirSync(awaited)",
        "readdirSync(fs.mkdtempDisposableSync('case-').path)",
        "readdirSync(disposable.path)",
        "readdirSync(defaultRoot)",
        "readdirSync(absent)",
        "readdirSync(fallback)",
        "readdirSync(ownedMember)",
    ] {
        assert_eq!(read.origin(needle).anchor, PathAnchor::Temp, "{needle}");
    }
    assert_eq!(
        read.origin("readdirSync(omitted)").anchor,
        PathAnchor::Absolute
    );
}

#[test]
fn mutable_escaping_and_unrepresentable_members_stay_unknown() {
    let read = read("members.mts");
    for needle in [
        "readdirSync(exposed.root)",
        "readdirSync(spread.root)",
        "readdirSync(computed.root)",
        "readdirSync(method.root)",
        "readdirSync(accessor.root)",
        "readdirSync(restRoot)",
        "readdirSync(computedRoot)",
        "readdirSync(arrayRoot)",
        "readdirSync(rewritten.root)",
        "readdirSync(escaped.root)",
        "readdirSync(mutable.root)",
        "readdirSync(missing().root)",
        "readdirSync(alias19.root)",
        "readdirSync(getter19().root)",
        "readdirSync(shadowed)",
        "readdirSync(fromUnknown)",
        "readdirSync(assignedRoot)",
    ] {
        assert_eq!(read.origin(needle), PathOrigin::unknown(), "{needle}");
    }
}
