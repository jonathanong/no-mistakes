/* class BlockCommentViewModel {}
   /* class NestedCommentViewModel {} */
*/
let plain = "class StringViewModel {}"
let raw = #"class RawStringViewModel {}"#
let multi = """
class MultilineStringViewModel {}
"""
@MainActor
class SafeViewModel {}
// no-mistakes-disable-next-line swift-viewmodel-main-actor
class SuppressedViewModel {}
class BrokenViewModel {}
