import XCTest
import Darwin

final class ClockTests: XCTestCase {
    func testPageSizeIsPositive() {
        XCTAssertGreaterThan(Darwin.getpagesize(), 0)
    }
}
