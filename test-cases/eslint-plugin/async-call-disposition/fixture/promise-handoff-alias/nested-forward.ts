import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
import { nestedAwait, nestedDrop, nestedOpaque } from './nested-consumers';
type Dependencies = { enqueue: typeof enqueueBulkCrawlReferralLinks };
export function forwardAwait(dependencies: Dependencies) {
  return nestedAwait({ ...dependencies, urlId: 'id' });
}
export function forwardDrop(dependencies: Dependencies) {
  return nestedDrop({ ...dependencies, urlId: 'id' });
}
export function forwardOpaque(dependencies: Dependencies) {
  return nestedOpaque({ ...dependencies, urlId: 'id' });
}
