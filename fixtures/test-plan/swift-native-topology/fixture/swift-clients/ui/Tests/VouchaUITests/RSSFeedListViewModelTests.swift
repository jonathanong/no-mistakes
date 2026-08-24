import XCTest
import VouchaFeatures

final class RSSFeedListViewModelTests: XCTestCase {
    func testRefresh() {
        let model = RSSFeedListViewModel()
        model.refresh()
    }
}
