use super::*;

#[derive(Debug)]
struct FormattingError;

impl std::fmt::Display for FormattingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Force a competing diagnostic exactly while the reporter formats its
        // error. The prefix and message must remain one complete stderr line.
        let marker = b"formatting marker\n";
        // Match detached diagnostic writes without re-entering stderr's formatter.
        assert_eq!(
            unsafe { nix::libc::write(2, marker.as_ptr().cast(), marker.len()) },
            marker.len() as isize
        );
        formatter.write_str("formatted failure")
    }
}

impl std::error::Error for FormattingError {}

#[test]
fn error_formatting_subprocess_helper() {
    if std::env::var_os("NO_MISTAKES_ERROR_FORMATTING_HELPER").is_none() {
        return;
    }
    let error = anyhow::Error::new(FormattingError);
    let code = report_error(&error, &CliErrorOutput::new(false));
    assert_eq!(code, ExitCode::from(2));
    std::process::exit(2);
}

#[test]
fn error_report_formats_the_complete_line_before_writing_stderr() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "invocation::tests::error_formatting::error_formatting_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_ERROR_FORMATTING_HELPER", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "formatting marker\nerror: formatted failure\n"
    );
}
