use super::*;
use std::path::PathBuf;

#[test]
fn zeitwerk_rel_underscores_nested_constants() {
    assert_eq!(zeitwerk_rel("UsersController"), "/users_controller.rb");
    assert_eq!(zeitwerk_rel("Admin::User"), "/admin/user.rb");
}

#[test]
fn underscore_splits_camel_case() {
    assert_eq!(underscore("WelcomeJob"), "welcome_job");
    assert_eq!(underscore("APIClient"), "api_client");
    assert_eq!(underscore("SSLError"), "ssl_error");
}

#[test]
fn under_app_requires_app_as_first_relative_component() {
    let root = PathBuf::from("/repo/rails");
    assert!(under_app(&root, &root.join("app/models/admin/user.rb")));
    assert!(!under_app(
        &root,
        &root.join("lib/app/models/admin/user.rb")
    ));
}
