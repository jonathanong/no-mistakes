use super::EdgeKind;

#[test]
fn server_route_edge_kind_display_matches_serialized_kebab_names() {
    assert_eq!(EdgeKind::ServerRoute.to_string(), "server-route");
    assert_eq!(EdgeKind::ClientCall.to_string(), "client-call");
    assert_eq!(
        serde_json::to_value(EdgeKind::ServerRoute).unwrap(),
        serde_json::json!("server-route")
    );
    assert_eq!(
        serde_json::to_value(EdgeKind::ClientCall).unwrap(),
        serde_json::json!("client-call")
    );
}
