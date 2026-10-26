export async function crawl(dependencies: any) {
  await dependencies.enqueueBulkCrawlReferralLinks([], {});
}
