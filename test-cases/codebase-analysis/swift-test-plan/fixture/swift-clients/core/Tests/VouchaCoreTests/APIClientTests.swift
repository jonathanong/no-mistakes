import XCTest
import VouchaCore
import VouchaAPI

final class APIClientTests: XCTestCase {
    func testLoadRSS() {
        let client = APIClient()
        client.loadRSS()
        _ = Endpoint<[RSSFeedItem]>.rssFeedItems("top")
    }
}
