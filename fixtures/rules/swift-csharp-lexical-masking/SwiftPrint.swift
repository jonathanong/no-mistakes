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
