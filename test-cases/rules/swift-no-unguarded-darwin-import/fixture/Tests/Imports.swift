import DarwinExtras

import Foundation; #if(canImport(Darwin))
import enum Darwin.ClockValue
#endif

#if canImport(Darwin)
import Darwin
#if DEBUG
import struct Darwin.timespec
#endif
#elseif canImport(Glibc)
import Glibc
#else
import Darwin
#endif

/*
#if DEBUG
import Darwin
*/
let sourceSnippet = #"""
import Darwin
#endif
"""#
let pattern = #/
import Darwin
/#

import func Darwin.getpagesize
@preconcurrency import Darwin
import /* system module */ `Darwin`
