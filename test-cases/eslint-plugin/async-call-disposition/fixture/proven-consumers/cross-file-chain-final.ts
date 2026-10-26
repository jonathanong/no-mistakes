export async function enqueueByHostname(rows: any[], dependencies: any) {
  await dependencies.computeHostnameRateLimitMs();
  await dependencies.enqueueBulkCrawlReferralLinks(rows);
}
