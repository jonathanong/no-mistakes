import { enqueueBulkCrawlReferralLinks as dflt } from '@queues/crawl-referral-links/enqueues';
type Options = { enqueue: typeof dflt; urlId: string };
declare function register(handler: () => Promise<void>): void;
declare function opaque(value: unknown): void;
export function nestedAwait(options: Options) {
  const handler = async () => {
    const dependencies = { enqueue: options.enqueue ?? dflt };
    await dependencies.enqueue([], undefined);
    await Promise.all([dependencies.enqueue([], undefined)]);
    const pending = dependencies.enqueue([], undefined);
    await pending;
  };
  register(handler);
}
export function nestedDrop(options: Options) {
  const handler = async () => {
    const dependencies = { enqueue: options.enqueue ?? dflt };
    dependencies.enqueue([], undefined);
  };
  register(handler);
}
export function nestedOpaque(options: Options) {
  const handler = async () => {
    const dependencies = { enqueue: options.enqueue ?? dflt };
    opaque(dependencies.enqueue([], undefined));
  };
  register(handler);
}
