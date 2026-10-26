export async function crawl(dependencies: any) {
  await dependencies.enqueueBulkCrawlReferralLinks([], {});
}

export async function crawlUnawaited(dependencies: any) {
  dependencies.enqueueBulkCrawlReferralLinks([], {});
}
