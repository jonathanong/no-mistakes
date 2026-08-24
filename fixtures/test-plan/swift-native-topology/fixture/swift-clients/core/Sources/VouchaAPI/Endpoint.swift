public struct Endpoint<Response> {
    public let path: String

    public static func rssFeedItems(_ feedType: String) -> Endpoint<[RSSFeedItem]> {
        Endpoint<[RSSFeedItem]>(path: "/api/v1/feeds/rss_feed_items/\(feedType)")
    }

    public static func posts(_ feedType: String) -> Endpoint<[Post]> {
        Endpoint<[Post]>(path: "/api/v1/feeds/posts/\(feedType)")
    }
}

public struct RSSFeedItem {}
public struct Post {}
