// no-mistakes-disable-file rust-max-lines-per-file: real header
// The apparent directives below are string contents, including embedded quotes.
const EXAMPLE: &str = r###"
" // no-mistakes-disable-line imaginary-raw
// no-mistakes-disable-next-line imaginary-raw-next
"###;
const NORMAL: &str = "\n// no-mistakes-disable-line imaginary-normal";
fn lifetime<'a>(value: &'a str) -> &'a str {
    value // no-mistakes-disable-line real-rust
}
/* nested /* comment */
// no-mistakes-disable-line imaginary-block
*/
// no-mistakes-disable-next-line real-next
fn genuine() {}
fn unicode_lifetime<'α>(value: &'α str) -> &'α str {
    value // no-mistakes-disable-line real-unicode-lifetime
}
const CHARACTER: char = '\'';
const BYTE: u8 = b'\\';
fn raw_identifier() {
    let r#type = 1; // no-mistakes-disable-line real-raw-identifier
}
const RAW_BYTE: &[u8] = br##"inner quote "
// no-mistakes-disable-line imaginary-raw-byte
"##;
const RAW_C: &std::ffi::CStr = cr##"inner quote "
// no-mistakes-disable-line imaginary-raw-c
"##;
