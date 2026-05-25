use super::*;

#[test]
fn normalize_plain_param() {
    assert_eq!(normalize_nextjs_pattern("/posts/:slug"), "/posts/:slug");
}

#[test]
fn normalize_zero_or_more() {
    assert_eq!(normalize_nextjs_pattern("/posts/:slug*"), "/posts/**");
}

#[test]
fn normalize_one_or_more() {
    assert_eq!(normalize_nextjs_pattern("/posts/:slug+"), "/posts/*");
}

#[test]
fn normalize_mixed_segments() {
    assert_eq!(
        normalize_nextjs_pattern("/content/:type/:path*"),
        "/content/:type/**"
    );
}

#[test]
fn normalize_root() {
    assert_eq!(normalize_nextjs_pattern("/"), "/");
}

#[test]
fn normalize_literal_only() {
    assert_eq!(normalize_nextjs_pattern("/about/team"), "/about/team");
}

#[test]
fn contained_literal_matches_param() {
    assert!(destination_contained_by_route(
        &["content", "posts"],
        &["content", ":type"]
    ));
}

#[test]
fn contained_param_matches_param() {
    assert!(destination_contained_by_route(&[":slug"], &[":id"]));
}

#[test]
fn contained_param_does_not_match_literal() {
    assert!(!destination_contained_by_route(&[":slug"], &["posts"]));
}

#[test]
fn contained_literal_must_equal() {
    assert!(!destination_contained_by_route(&["posts"], &["reviews"]));
}

#[test]
fn contained_route_double_star_accepts_all() {
    assert!(destination_contained_by_route(
        &["content", "posts", "hello"],
        &["content", ":type", "**"]
    ));
}

#[test]
fn contained_route_star_requires_segments() {
    assert!(destination_contained_by_route(
        &["content", "posts", "hello"],
        &["content", ":type", "*"]
    ));
}

#[test]
fn contained_route_star_rejects_empty() {
    assert!(!destination_contained_by_route(
        &["content"],
        &["content", "*"]
    ));
}

#[test]
fn contained_dest_double_star_vs_route_double_star() {
    assert!(destination_contained_by_route(
        &["content", "**"],
        &["content", "**"]
    ));
}

#[test]
fn contained_dest_star_vs_route_star() {
    assert!(destination_contained_by_route(
        &["content", "*"],
        &["content", "*"]
    ));
}

#[test]
fn contained_dest_star_vs_route_double_star() {
    assert!(destination_contained_by_route(
        &["content", "*"],
        &["content", "**"]
    ));
}

#[test]
fn contained_dest_double_star_rejects_route_star() {
    assert!(!destination_contained_by_route(
        &["content", "**"],
        &["content", "*"]
    ));
}

#[test]
fn contained_dest_double_star_vs_literal() {
    assert!(!destination_contained_by_route(&["**"], &["content"]));
}

#[test]
fn contained_dest_star_vs_literal() {
    assert!(!destination_contained_by_route(&["*"], &["content"]));
}

#[test]
fn contained_dest_wildcard_not_last_rejected() {
    assert!(!destination_contained_by_route(
        &["**", "extra"],
        &["content"]
    ));
}

#[test]
fn contained_shorter_dest_rejected() {
    assert!(!destination_contained_by_route(
        &["content"],
        &["content", ":type"]
    ));
}

#[test]
fn contained_longer_dest_rejected() {
    assert!(!destination_contained_by_route(
        &["content", "posts", "extra"],
        &["content", ":type"]
    ));
}

#[test]
fn contained_empty_vs_empty() {
    assert!(destination_contained_by_route(&[], &[]));
}

#[test]
fn contained_dest_double_star_vs_route_double_star_after_literal() {
    assert!(destination_contained_by_route(
        &["content", "posts", "**"],
        &["content", ":type", "**"]
    ));
}

#[test]
fn expand_basic() {
    let routes = vec![
        Route {
            file: PathBuf::from("/app/content/[type]/[[...slug]]/page.tsx"),
            pattern: "/content/:type/**".to_string(),
        },
        Route {
            file: PathBuf::from("/app/page.tsx"),
            pattern: "/".to_string(),
        },
    ];
    let rewrites = vec![RewriteRule {
        source: "/posts/:slug*".to_string(),
        destination: "/content/posts/:slug*".to_string(),
    }];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert_eq!(virtual_routes.len(), 1);
    assert_eq!(virtual_routes[0].pattern, "/posts/**");
    assert_eq!(
        virtual_routes[0].file,
        PathBuf::from("/app/content/[type]/[[...slug]]/page.tsx")
    );
}

#[test]
fn expand_multiple_rewrites_same_dest() {
    let routes = vec![Route {
        file: PathBuf::from("/app/content/[type]/[[...slug]]/page.tsx"),
        pattern: "/content/:type/**".to_string(),
    }];
    let rewrites = vec![
        RewriteRule {
            source: "/posts/:slug*".to_string(),
            destination: "/content/posts/:slug*".to_string(),
        },
        RewriteRule {
            source: "/reviews/:slug*".to_string(),
            destination: "/content/reviews/:slug*".to_string(),
        },
    ];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert_eq!(virtual_routes.len(), 2);
    assert_eq!(virtual_routes[0].pattern, "/posts/**");
    assert_eq!(virtual_routes[1].pattern, "/reviews/**");
}

#[test]
fn expand_no_matching_dest() {
    let routes = vec![Route {
        file: PathBuf::from("/app/page.tsx"),
        pattern: "/".to_string(),
    }];
    let rewrites = vec![RewriteRule {
        source: "/posts/:slug".to_string(),
        destination: "/content/posts/:slug".to_string(),
    }];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert!(virtual_routes.is_empty());
}

#[test]
fn expand_self_referencing_skipped() {
    let routes = vec![Route {
        file: PathBuf::from("/app/posts/page.tsx"),
        pattern: "/posts".to_string(),
    }];
    let rewrites = vec![RewriteRule {
        source: "/posts".to_string(),
        destination: "/posts".to_string(),
    }];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert!(virtual_routes.is_empty());
}

#[test]
fn expand_empty_rewrites() {
    let routes = vec![Route {
        file: PathBuf::from("/app/page.tsx"),
        pattern: "/".to_string(),
    }];
    let virtual_routes = expand_rewrites(&[], &routes);
    assert!(virtual_routes.is_empty());
}

#[test]
fn expand_empty_routes() {
    let rewrites = vec![RewriteRule {
        source: "/posts/:slug".to_string(),
        destination: "/content/posts/:slug".to_string(),
    }];
    let virtual_routes = expand_rewrites(&rewrites, &[]);
    assert!(virtual_routes.is_empty());
}

#[test]
fn expand_deduplicates() {
    let routes = vec![Route {
        file: PathBuf::from("/app/content/[type]/[[...slug]]/page.tsx"),
        pattern: "/content/:type/**".to_string(),
    }];
    let rewrites = vec![
        RewriteRule {
            source: "/posts/:slug*".to_string(),
            destination: "/content/posts/:slug*".to_string(),
        },
        RewriteRule {
            source: "/posts/:path*".to_string(),
            destination: "/content/posts/:path*".to_string(),
        },
    ];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert_eq!(virtual_routes.len(), 1);
}

#[test]
fn expand_as_tuples() {
    let routes = vec![(
        PathBuf::from("/app/content/[type]/[[...slug]]/page.tsx"),
        "/content/:type/**".to_string(),
    )];
    let rewrites = vec![RewriteRule {
        source: "/posts/:slug*".to_string(),
        destination: "/content/posts/:slug*".to_string(),
    }];
    let virtual_routes = expand_rewrites_as_tuples(&rewrites, &routes);
    assert_eq!(virtual_routes.len(), 1);
    assert_eq!(virtual_routes[0].1, "/posts/**");
}

#[test]
fn expand_single_param_rewrite() {
    let routes = vec![Route {
        file: PathBuf::from("/app/content/docs/[section]/page.tsx"),
        pattern: "/content/docs/:section".to_string(),
    }];
    let rewrites = vec![RewriteRule {
        source: "/documentation/:section".to_string(),
        destination: "/content/docs/:section".to_string(),
    }];
    let virtual_routes = expand_rewrites(&rewrites, &routes);
    assert_eq!(virtual_routes.len(), 1);
    assert_eq!(virtual_routes[0].pattern, "/documentation/:section");
}
