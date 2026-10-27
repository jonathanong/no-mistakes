//! no-mistakes-disable-file rust-max-lines-per-file: ineffective inner documentation
//! ```rust
//! no-mistakes-disable-file fenced-backtick
//! ``
//! no-mistakes-disable-line still-fenced-short-close
//! ```
//! no-mistakes-disable-next-line rust-max-lines-per-file
fn inner_target() {}
/// no-mistakes-disable-file rust-max-lines-per-file: ineffective even after code
/// no-mistakes-disable-line rust-max-lines-per-file
/// no-mistakes-disable-next-line rust-max-lines-per-file
fn outer_target() {}
/// ~~~~rust
/// no-mistakes-disable-file fenced-tilde
/// ```
/// no-mistakes-disable-line still-fenced-other-marker
/// ~~~
/// no-mistakes-disable-line still-fenced-short-tilde
/// ~~~~
/// no-mistakes-disable-next-line rust-max-lines-per-file
fn after_tilde() {}
// Ordinary comments must keep working beside documentation.
// no-mistakes-disable-next-line rust-max-lines-per-file
fn plain_target() {}
/// ```rust
/// no-mistakes-disable-file unclosed-example
fn group_boundary() {}
/// no-mistakes-disable-line rust-max-lines-per-file: a new doc group resets fences
/// ```rust
//! no-mistakes-disable-line rust-max-lines-per-file: inner docs start a separate group
fn kind_boundary() {}
const STRING: &str = "/// no-mistakes-disable-file string-example";
const RAW: &str = r#"//! no-mistakes-disable-file raw-example"#;
/* /// no-mistakes-disable-file block-example */
//// no-mistakes-disable-line ordinary-four-slashes
/// no-mistakes-disable-unknown unsupported-doc-directive
/// no-mistakes-disable-line : missing-rule
/// no-mistakes-disable-line unknown-doc-rule: placement wins over unknown target
