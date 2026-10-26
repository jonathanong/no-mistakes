import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
export async function consume(options: { enqueue: typeof enqueueBulkCrawlReferralLinks }) {
  await options.enqueue([], undefined);
}
export function drop(options: { enqueue: typeof enqueueBulkCrawlReferralLinks }) {
  options.enqueue([], undefined);
}
