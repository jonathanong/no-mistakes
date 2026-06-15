use super::*;

#[test]
fn resolve_format_prefers_json_then_explicit_then_tty() {
    assert_eq!(
        resolve_format(true, Some(Format::Human), true),
        Format::Json
    );
    assert_eq!(resolve_format(false, Some(Format::Md), false), Format::Md);
    assert_eq!(resolve_format(false, None, true), Format::Human);
    assert_eq!(resolve_format(false, None, false), Format::Json);
}
