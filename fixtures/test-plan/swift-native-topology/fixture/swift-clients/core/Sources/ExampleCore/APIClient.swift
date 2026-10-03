import ExampleAPI

public final class APIClient {
    public init() {}

    public func loadRSS() {
        _ = Endpoint<[RSSFeedItem]>.rssFeedItems("top")
    }
}
