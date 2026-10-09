use super::collect;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-mapped-context",
    ).join(name)
}

#[test]
fn only_sloppy_ordinary_simple_parameter_functions_are_mapped() {
    let path = fixture("sloppy.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let mapped = crate::ast::with_program(&path, &source, |program, _| collect(program)).unwrap();
    for name in ["simple", "duplicate", "nestedSloppy"] {
        let start = source.find(&format!("function {name}(")).unwrap() as u32;
        assert!(mapped.contains(&start), "missing mapped function {name}");
    }
    assert_eq!(
        mapped.len(),
        3,
        "strict or non-simple functions must not map"
    );
    for name in [
        "defaults",
        "destructured",
        "rest",
        "ownStrict",
        "nestedStrict",
        "nestedStrictArrow",
        "nestedOwnStrictArrow",
        "heritage",
        "nestedClass",
    ] {
        let start = source.find(&format!("function {name}(")).unwrap() as u32;
        assert!(
            !mapped.contains(&start),
            "unexpected mapped function {name}"
        );
    }
}

#[test]
fn modules_and_program_strict_directives_disable_mapping() {
    for name in ["module.mts", "strict-script.cjs"] {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let mapped =
            crate::ast::with_program(&path, &source, |program, _| collect(program)).unwrap();
        assert!(mapped.is_empty(), "{name} must have no mapped functions");
    }
}
