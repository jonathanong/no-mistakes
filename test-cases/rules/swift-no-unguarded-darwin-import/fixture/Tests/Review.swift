#if DEBUG
import Darwin
#elseif !os(macOS)
@_spi(Private) import Darwin
#elseif os(iOS)
import Darwin
#else
import Darwin
#endif; import Darwin

#if os(macOS)
#if DEBUG
import Darwin
#else
import Darwin
#endif
#endif

#if os(Linux)
import Darwin
#endif
#if canImport(Darwin) || DEBUG
import Darwin
#endif

import Foundation; import Darwin
@_spi(Private) import
    Darwin
import
    func
    Darwin.getpagesize

let patternWithImport = /; import Darwin; #endif/
let patternWithEscapedSlash = /; #if DEBUG \//
import Darwin
