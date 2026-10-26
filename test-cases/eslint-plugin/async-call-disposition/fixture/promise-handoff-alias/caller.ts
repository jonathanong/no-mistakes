import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
import { consume, drop } from './consumer';
declare function opaque(value: unknown): void;
const enqueueAndRecord: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  void entries;
  return enqueueBulkCrawlReferralLinks(entries, options);
};
void consume({ enqueue: enqueueAndRecord });
// A matching annotation cannot hide a promise discarded by the receiving body.
const dropped: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  return enqueueBulkCrawlReferralLinks(entries, options);
};
drop({ enqueue: dropped });
const unknownConsumer: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  return enqueueBulkCrawlReferralLinks(entries, options);
};
opaque({ enqueue: unknownConsumer });
