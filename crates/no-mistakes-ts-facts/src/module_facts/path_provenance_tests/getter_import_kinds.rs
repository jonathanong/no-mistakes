use super::*;

#[test]
fn type_only_imports_cannot_supply_runtime_environment_getters() {
    let read = read("getter-import-kinds.mts");
    assert_eq!(
        read.origin("readdirSync(typeOnly())"),
        PathOrigin::unknown()
    );
    assert!(read.origin("readdirSync(runtime())").getter.is_some());
}
