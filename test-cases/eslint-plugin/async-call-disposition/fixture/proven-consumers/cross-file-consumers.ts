export async function processReconciliationPage(callback: () => Promise<unknown>) {
  await callback();
}
export async function processReferrals(dependencies: { enqueueBulkCrawlReferralLinks: () => Promise<unknown> }) {
  await dependencies.enqueueBulkCrawlReferralLinks();
}
export function unknownConsumer(callback: unknown) { void callback; }
