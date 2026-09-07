import XCTest
import App

final class Alpha: XCTestCase {
    func testValue() {
        XCTAssertEqual(value(), 42)
    }
}
