import XCTest

#if os(iOS) && !targetEnvironment(macCatalyst)
import UIKit
#endif

#if os(macOS) && canImport(AppKit)
import AppKit
#endif

#if os(tvOS) && !targetEnvironment(simulator)
import UIKit
#endif

#if os(watchOS) && canImport(Security)
import Security
#endif

#if os(visionOS) && !targetEnvironment(simulator)
import UIKit
#endif

#if os(iOS) &&
    !targetEnvironment(macCatalyst)
import UIKit
#endif

#if (os(iOS) || os(macOS)) && !targetEnvironment(macCatalyst)
import Security
#endif

#if canImport(Security) && !targetEnvironment(simulator)
import Security
#endif

final class GuardTests: XCTestCase {
    func testGuardedImports() {}
}
