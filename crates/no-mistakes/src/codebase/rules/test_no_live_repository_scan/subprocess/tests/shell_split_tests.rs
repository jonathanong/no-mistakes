use super::{splits, ArgumentValue, Token};
use serde_json::Value;

fn scan_directories(text: &str) -> Vec<Option<String>> {
    splits(&[ArgumentValue::Scalar(Value::String(text.to_owned()))])
        .into_iter()
        .find(|invocation| matches!(&invocation.executable, Token::Known(name) if name == "widget-audit"))
        .expect("expected a scan command")
        .shell_dirs
        .expect("expected shell directories")
}

#[test]
fn and_keeps_success_from_the_first_cd_or_its_fallback() {
    assert_eq!(
        scan_directories("cd /workspace || cd /repo && widget-audit check"),
        [Some("/repo".to_owned()), Some("/workspace".to_owned())]
    );
}

#[test]
fn or_keeps_the_directory_where_a_cd_failed() {
    assert_eq!(
        scan_directories("cd /repo && cd fixtures || widget-audit check"),
        [None, Some("/repo".to_owned())]
    );
}

#[test]
fn semicolon_combines_prior_success_and_failure() {
    assert_eq!(
        scan_directories("cd /repo && check-preflight; widget-audit check"),
        [None, Some("/repo".to_owned())]
    );
}

#[test]
fn semicolon_after_cd_retains_paths_that_skipped_it() {
    assert_eq!(
        scan_directories("cd /repo && cd fixtures; widget-audit check"),
        [None, Some("fixtures".to_owned())]
    );
}
