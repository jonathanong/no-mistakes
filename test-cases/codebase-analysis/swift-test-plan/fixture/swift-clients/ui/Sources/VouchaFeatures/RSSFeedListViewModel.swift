import VouchaCore
import VouchaAPI

public final class RSSFeedListViewModel {
    let client = APIClient()

    public func refresh() {
        client.loadRSS()
        _ = Endpoint<[RSSFeedItem]>.rssFeedItems("top")
    }
}
