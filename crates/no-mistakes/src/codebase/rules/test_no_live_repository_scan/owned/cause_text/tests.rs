use super::super::{Cause, CauseKind};
use std::path::Path;

#[test]
fn empty_expression_names_the_argument_kind() {
    let cause = Cause::new(CauseKind::LivePath, "/repo/src/a.mts".into(), 4, "");
    let text = super::detail(&cause, Path::new("/repo"));
    assert!(
        text.contains("src/a.mts:4 passes an argument that is a live path"),
        "{text}"
    );
}

#[test]
fn empty_specifier_names_the_import_form() {
    let cause = Cause::new(CauseKind::NamespaceImport, "/repo/src/a.mts".into(), 4, "");
    let text = super::detail(&cause, Path::new("/repo"));
    assert!(
        text.contains("src/a.mts:4 uses a namespace import"),
        "{text}"
    );
}
