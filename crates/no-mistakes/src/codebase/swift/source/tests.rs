use super::*;

#[test]
fn comment_stripping_preserves_comment_markers_inside_strings() {
    let source = r#"
        let site = "https://example.com/feed"
        static let rss = Endpoint(path: "/api/v1/feeds/rss_feed_items/\(feedType)")
        // Endpoint(path: "/api/v1/commented")
        let marker = "not /* a comment */"
        /* Endpoint(path: "/api/v1/blocked")
           Endpoint(path: "/api/v1/also-blocked") */
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

#[test]
fn compiled_patterns_and_keyword_table_are_reused() {
    assert!(std::ptr::eq(swift_import_regex(), swift_import_regex()));
    assert!(std::ptr::eq(
        swift_declaration_regex(),
        swift_declaration_regex()
    ));
    assert!(std::ptr::eq(
        swift_reference_regex(),
        swift_reference_regex()
    ));
    assert!(std::ptr::eq(swift_function_regex(), swift_function_regex()));
    assert!(std::ptr::eq(swift_property_regex(), swift_property_regex()));
    assert!(std::ptr::eq(
        swift_endpoint_path_regex(),
        swift_endpoint_path_regex()
    ));
    assert!(std::ptr::eq(
        swift_interpolation_regex(),
        swift_interpolation_regex()
    ));
    assert!(std::ptr::eq(
        swift_reference_keywords(),
        swift_reference_keywords()
    ));
}
