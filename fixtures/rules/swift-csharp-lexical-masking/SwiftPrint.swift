/* print("block comment")
   /* print("nested block comment") */
*/
let url = "https://example.test/print()"
let escaped = "say \\\"print()\\\""
let raw = #"print()"#
let multi = """
print("multiline string")
"""
let heading = "café"
print("real call")
let interpolated = "result: \(print("interpolated call"))"
// no-mistakes-disable-next-line swift-no-raw-print
print("suppressed call")
// Quote inside an extended regex must not open a string.
let quote = #/"/#
print("after regex")
// Matching-hash escapes keep later quotes inside the literal.
let escapedHash = #"abc \#"# print(fake) end"#
print("after escape")
// Keep the interpolation closer so print cannot join a later '('.
let named = "\(print)"
(f)()
