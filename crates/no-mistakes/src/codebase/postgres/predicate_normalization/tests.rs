#[test]
fn literals_and_quoted_names_keep_case_and_whitespace() {
    let normalize = super::normalize;
    assert_eq!(normalize("'"), "'");
    assert_eq!(normalize("STATUS = 'idle'"), normalize("status = 'idle'"));
    assert_ne!(normalize("status = 'IDLE'"), normalize("status = 'idle'"));
    assert_ne!(
        normalize("\"Status\" = 'a  b'"),
        normalize("\"status\" = 'a b'")
    );
    assert_ne!(
        normalize("status = $$IDLE$$"),
        normalize("status = $$idle$$")
    );
}
