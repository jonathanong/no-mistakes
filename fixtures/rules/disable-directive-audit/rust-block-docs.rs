/*! no-mistakes-disable-file rust-max-lines-per-file: inner block docs are ineffective */
/** no-mistakes-disable-line outer-single */
/**
 * no-mistakes-disable-next-line outer-decorated
 */
fn outer_target() {}
/*!
no-mistakes-disable-line inner-multiline
*/
/** ```rust
 * no-mistakes-disable-file fenced-backtick
 * ``
 * no-mistakes-disable-line short-close
 * ```
 * no-mistakes-disable-line after-backtick
 */
/** ~~~~rust
 * no-mistakes-disable-file fenced-tilde
 * ```
 * no-mistakes-disable-line other-marker
 * ~~~
 * no-mistakes-disable-line short-tilde
 * ~~~~
 * no-mistakes-disable-line after-tilde
 */
/** ```rust */
// Rustdoc joins adjacent attributes across blank lines and ordinary comments.

/** no-mistakes-disable-line cross-block-example */
/// no-mistakes-disable-line cross-line-example
/** ``` */
/** no-mistakes-disable-line after-cross-block */
/** ```rust
 * no-mistakes-disable-line unclosed-example
 */
fn fence_boundary() {}
/** no-mistakes-disable-line after-code-boundary */
/** ```rust */
/*! no-mistakes-disable-line inner-kind-boundary */
fn kind_boundary() {}
/* Ordinary nested documentation-looking comments are not Rust documentation.
 /** no-mistakes-disable-file nested-outer */
 /*! no-mistakes-disable-file nested-inner */
*/
/*** no-mistakes-disable-line ordinary-three-stars */
const TEXT: &str = "/** no-mistakes-disable-line string-example */";
const RAW: &str = r###"/*! no-mistakes-disable-file raw-example */"###;
fn lifetime<'a>(value: &'a str) -> &'a str { value }
/** no-mistakes-disable-line after-lifetime */
// no-mistakes-disable-next-line rust-max-lines-per-file
fn ordinary_target() {}
