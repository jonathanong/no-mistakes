use super::*;

#[test]
fn comment_stripping_preserves_comment_markers_inside_strings() {
    let source = r#"
        let site = "https://example.com/feed"
        static let rss = Endpoint(path: "/api/v1/feeds/rss_feed_items/\(feedType)")
        // Endpoint(path: "/api/v1/commented")
        let marker = "not /* a comment */"
        /* Endpoint(path: "/api/v1/blocked") */
    "#;

    let stripped = strip_comments(source);
    assert!(stripped.contains(r#""https://example.com/feed""#));
    assert!(stripped.contains(r#""/api/v1/feeds/rss_feed_items/\(feedType)""#));
    assert!(stripped.contains(r#""not /* a comment */""#));
    assert_eq!(
        extract_endpoint_paths(&stripped),
        vec!["/api/v1/feeds/rss_feed_items/*".to_string()]
    );
}
