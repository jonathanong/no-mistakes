use super::super::arguments::PathValueKind;
use super::*;

#[test]
fn temp_factory_return_kind_is_not_the_module_family() {
    let read = read("temp-factory-return-kinds.mts");
    for expression in [
        "readdirSync(mkdtempSync('sync-'))",
        "readdirSync(await mkdtempSync('await-sync-'))",
        "readdirSync(await makeTemp('await-promise-'))",
        "readdirSync(await pending)",
        "readdirSync(await alias)",
        "readdirSync(await renamed)",
        "readdirSync(nodeFs.mkdtempSync('member-sync-'))",
        "readdirSync(await promises.mkdtemp('await-ns-'))",
        "readdirSync(await nodeFs.promises.mkdtemp('await-member-promise-'))",
    ] {
        let origin = read.origin(expression);
        assert_eq!(origin.anchor, PathAnchor::Temp, "{expression}: {origin:?}");
        assert_eq!(
            origin.value_kind,
            PathValueKind::Scalar,
            "{expression}: {origin:?}"
        );
    }
    for expression in [
        "readdirSync(mkdtempSync('sync-option-', 'buffer'))",
        "readdirSync(await makeTemp('await-promise-option-', 'buffer'))",
        "readdirSync(await buffered)",
        "readdirSync(await bufferAlias)",
    ] {
        let origin = read.origin(expression);
        assert_eq!(origin.anchor, PathAnchor::Temp, "{expression}: {origin:?}");
        assert_eq!(
            origin.value_kind,
            PathValueKind::Mutable,
            "{expression}: {origin:?}"
        );
    }
    for expression in [
        "readdirSync(makeTemp('promise-'))",
        "readdirSync(pending)",
        "readdirSync(await filledBuffer)",
        "readdirSync(await filledAlias)",
        "readdirSync(await letPending)",
        "readdirSync(await aliasOfLet)",
        "readdirSync(await pending.path)",
        "readdirSync(await hop0)",
        "readdirSync(mkdtemp('callback-one-'))",
        "readdirSync(mkdtemp('callback-two-', () => {}))",
        "readdirSync(await mkdtemp('await-callback-', () => {}))",
        "readdirSync(promises.mkdtemp('ns-promise-'))",
        "readdirSync(nodeFs.mkdtemp('member-callback-'))",
        "readdirSync(nodeFs.promises.mkdtemp('member-promise-'))",
        "readdirSync(replacedSync('written-sync'))",
        "readdirSync(await nodeFs['mkdtemp']('computed-member'))",
    ] {
        let origin = read.origin(expression);
        assert_eq!(
            origin.anchor,
            PathAnchor::Unknown,
            "{expression}: {origin:?}"
        );
    }
    for expression in [
        "readdirSync(await directoryName)",
        "readdirSync(await directoryAlias)",
    ] {
        let awaited_literal = read.origin(expression);
        assert_eq!(
            awaited_literal.anchor,
            PathAnchor::Relative,
            "{expression}: {awaited_literal:?}"
        );
        assert_eq!(awaited_literal.segments, ["not-a-call"]);
        assert_eq!(awaited_literal.value_kind, PathValueKind::Scalar);
    }
}
