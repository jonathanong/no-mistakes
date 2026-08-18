use super::*;

#[test]
fn zeitwerk_rel_underscores_nested_constants() {
    assert_eq!(zeitwerk_rel("UsersController"), "/users_controller.rb");
    assert_eq!(zeitwerk_rel("Admin::User"), "/admin/user.rb");
}

#[test]
fn underscore_splits_camel_case() {
    assert_eq!(underscore("WelcomeJob"), "welcome_job");
}
