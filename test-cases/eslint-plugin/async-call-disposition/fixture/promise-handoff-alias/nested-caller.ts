import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
import { forwardAwait, forwardDrop, forwardOpaque } from './nested-forward';
const awaited: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  return enqueueBulkCrawlReferralLinks(entries, options);
};
forwardAwait({ enqueue: awaited });
// Nested copies must not erase the producer when the terminal promise is dropped.
const dropped: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  return enqueueBulkCrawlReferralLinks(entries, options);
};
forwardDrop({ enqueue: dropped });
const opaque: typeof enqueueBulkCrawlReferralLinks = (entries, options) => {
  return enqueueBulkCrawlReferralLinks(entries, options);
};
forwardOpaque({ enqueue: opaque });
