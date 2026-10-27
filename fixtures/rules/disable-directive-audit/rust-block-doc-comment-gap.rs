/** ```rust */
/* Rustdoc joins attributes across ordinary multiline comments, including nested
   comment framing; these physical gaps must not close the example fence.
   /* no-mistakes-disable-line nested-ordinary-example */
*/
/** no-mistakes-disable-line still-fenced-after-ordinary-block */
/** ``` */
/** no-mistakes-disable-line after-ordinary-block-fence */
fn target() {}
