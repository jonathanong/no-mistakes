use crate::playwright::playwright_urls::literals::extract_href_from_selector;

#[test]
fn test_extract_href_from_selector() {
    // Double quotes
    assert_eq!(
        extract_href_from_selector("a[href=\"/users/42\"]"),
        Some("/users/42".to_string())
    );
    assert_eq!(
        extract_href_from_selector("a[href=\"https://example.com\"]"),
        Some("https://example.com".to_string())
    );

    // Single quotes
    assert_eq!(
        extract_href_from_selector("a[href='/users/42']"),
        Some("/users/42".to_string())
    );
    assert_eq!(
        extract_href_from_selector("a[href='http://example.com']"),
        Some("http://example.com".to_string())
    );

    // Non-candidate URLs (from is_candidate_url)
    assert_eq!(
        extract_href_from_selector("a[href=\"javascript:void(0)\"]"),
        None
    );
    assert_eq!(extract_href_from_selector("a[href=\"about:blank\"]"), None);
    assert_eq!(extract_href_from_selector("a[href=\"users/42\"]"), None);

    // No href
    assert_eq!(extract_href_from_selector("a.button"), None);
}
