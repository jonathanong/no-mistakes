import XCTest
import ExampleFeatures

final class RSSFeedListViewModelTests: XCTestCase {
    func testRefresh() {
        let model = RSSFeedListViewModel()
        model.refresh()
    }
}
