import XCTest
import ExampleCore
import ExampleAPI

final class APIClientTests: XCTestCase {
    func testLoadRSS() {
        let client = APIClient()
        client.loadRSS()
        _ = Endpoint<[RSSFeedItem]>.rssFeedItems("top")
    }
}
