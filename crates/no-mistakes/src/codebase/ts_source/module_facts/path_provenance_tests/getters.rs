use super::*;

#[test]
fn imported_getter_paths_are_deferred_but_captured_and_argument_calls_are_not() {
    let read = read("imported-getter-paths.mjs");
    let root = read.origin("readdirSync(join(getLocalDir(),");
    assert_eq!(
        root.getter,
        Some(read.source.find("getLocalDir(),").unwrap() as u32)
    );
    assert_eq!(root.segments, ["table"]);
    assert_eq!(read.origin("readdirSync(captured)"), PathOrigin::unknown());
    assert_eq!(
        read.origin("readdirSync(getLocalDir(\"argument\"))"),
        PathOrigin::unknown()
    );
    let offset = read.source.find("function pure()").unwrap() as u32;
    assert_eq!(
        read.provenance().env_getters[&offset].env.as_deref(),
        Some("X")
    );
    let absolute = read.source.find("function absolute()").unwrap() as u32;
    assert!(!read.provenance().env_getters.contains_key(&absolute));
    let impure = read.source.find("function impure()").unwrap() as u32;
    assert!(!read.provenance().env_getters.contains_key(&impure));
}
