import XCTest
import App

final class Beta: XCTestCase {
    func testValue() {
        XCTAssertEqual(value(), 42)
    }
}
