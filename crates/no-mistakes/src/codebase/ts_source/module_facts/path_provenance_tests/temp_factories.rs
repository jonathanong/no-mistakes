use super::*;

#[test]
fn classic_temp_roots_require_stable_node_bindings() {
    let read = read("temp-factory-identity.mts");
    for expression in [
        "readdirSync(owned('case-'))",
        "readdirSync(fs.mkdtempSync('case-'))",
    ] {
        assert_eq!(
            read.origin(expression).anchor,
            PathAnchor::Temp,
            "{expression}"
        );
    }
    for expression in [
        "readdirSync(owned('shadow-'))",
        "readdirSync(wrong('case-'))",
        "readdirSync(impossible('case-'))",
        "readdirSync(promises.mkdtemp('case-'))",
        "readdirSync(namedPromises.mkdtemp('case-'))",
        "readdirSync(nodeFs.promises.mkdtemp('case-'))",
        "readdirSync(promises.mkdtempSync('case-'))",
        "readdirSync(promisesDefault.mkdtempSync('case-'))",
        "readdirSync(nodeFs.promises.mkdtempSync('case-'))",
        "readdirSync(wrongFs.promises.mkdtemp('case-'))",
    ] {
        assert_eq!(
            read.origin(expression).anchor,
            PathAnchor::Unknown,
            "{expression}"
        );
    }
    let read = super::read("temp-factory-mutated.mts");
    for expression in [
        "readdirSync(escaped.mkdtempSync('case-'))",
        "readdirSync(overwritten.mkdtempSync('case-'))",
        "readdirSync(mkdtempSync('case-'))",
    ] {
        assert_eq!(
            read.origin(expression).anchor,
            PathAnchor::Unknown,
            "{expression}"
        );
    }
    for name in [
        "assign",
        "computed",
        "second-argument",
        "opaque-call",
        "opaque-call-other",
        "fake-spy",
    ] {
        let read = super::read(&format!("temp-factory-escapes/{name}.mts"));
        let expression = "readdirSync(mkdtempSync('case-'))";
        assert_eq!(
            read.origin(expression).anchor,
            PathAnchor::Unknown,
            "{name}"
        );
    }
}
