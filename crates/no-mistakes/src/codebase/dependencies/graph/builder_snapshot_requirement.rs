fn require_playwright_route_snapshot(
    snapshot: Option<&crate::playwright::fsutil::VisiblePathSnapshot>,
) -> Result<&crate::playwright::fsutil::VisiblePathSnapshot> {
    snapshot
        .ok_or_else(|| anyhow::anyhow!("Playwright graph plan requires a visible-path snapshot"))
}
