import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
interface Dependencies { enqueue: typeof enqueueBulkCrawlReferralLinks }
type Options = Dependencies & { urlId?: string };
export function adopt(options: Options) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
export function drop(options: Options) { options.enqueue([], undefined); }
// These contracts deliberately exceed the bounded source projection.
export function overlap(options: Dependencies & { enqueue: typeof enqueueBulkCrawlReferralLinks }) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
export function missing(options: {} & { urlId?: string }) { void options; }
export function opaque(options: Dependencies & Unknown) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
export function union(options: Dependencies & ({ a: string } | { b: string })) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
export function indexed(options: Dependencies & { [key: string]: unknown }) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
interface Base { a?: string }
interface Extended extends Base { b?: string }
export function extended(options: Dependencies & Extended) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
type Generic<T> = { value: T };
export function generic(options: Dependencies & Generic<string>) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
declare const key: unique symbol;
export function computed(options: Dependencies & { [key]: string }) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
// Cyclic source contracts remain unproven even if a type checker rejects them too.
type Cycle = Cycle & {};
export function cyclic(options: Dependencies & Cycle) { const handler = async () => { await options.enqueue([], undefined); }; return handler; }
