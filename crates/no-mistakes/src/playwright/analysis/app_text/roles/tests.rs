use super::first_role_token;

#[test]
fn first_role_token_skips_abstract_or_unknown_roles() {
    assert_eq!(
        first_role_token("roletype custom button").as_deref(),
        Some("button")
    );
    assert_eq!(first_role_token("roletype custom"), None);
}
