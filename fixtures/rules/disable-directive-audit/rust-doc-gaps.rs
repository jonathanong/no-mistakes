/// ```rust

/// no-mistakes-disable-line hidden-after-blank
// Rustdoc joins documentation attributes across ordinary comments, too.
/// no-mistakes-disable-file hidden-after-comment
// no-mistakes-disable-line ordinary-comment-still-audited
/// ```
fn previous() {} /// no-mistakes-disable-next-line trailing-doc
fn target() {}
/// no-mistakes-disable-line 	whitespace-rule: accepted trim whitespace
/// no-mistakes-disable-line	tab-separator: rejected by the directive grammar
/// no-mistakes-disable-next-line  	spaced-rule	with reason
fn after_whitespace() {}
