import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
export function consume(options: { enqueue: typeof enqueueBulkCrawlReferralLinks }): { enqueue: typeof enqueueBulkCrawlReferralLinks } {
  return options;
}
export function drop(options: { enqueue: typeof enqueueBulkCrawlReferralLinks }) {
  options.enqueue([], undefined);
}
