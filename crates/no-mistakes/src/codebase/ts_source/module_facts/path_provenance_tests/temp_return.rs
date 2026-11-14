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
        "readdirSync(mkdtemp('callback-one-'))",
        "readdirSync(mkdtemp('callback-two-', () => {}))",
        "readdirSync(await mkdtemp('await-callback-', () => {}))",
        "readdirSync(promises.mkdtemp('ns-promise-'))",
        "readdirSync(nodeFs.mkdtemp('member-callback-'))",
        "readdirSync(nodeFs.promises.mkdtemp('member-promise-'))",
    ] {
        let origin = read.origin(expression);
        assert_eq!(
            origin.anchor,
            PathAnchor::Unknown,
            "{expression}: {origin:?}"
        );
    }
}
