// Keep comments and blank lines: this fixture distinguishes physical and code lines.
/// The documentation does not consume the code-line budget.
/* A block comment can span
   /* nested comments */
   multiple physical lines. */

pub fn value() -> u8 {
    7 // A trailing comment shares this code line.
}
