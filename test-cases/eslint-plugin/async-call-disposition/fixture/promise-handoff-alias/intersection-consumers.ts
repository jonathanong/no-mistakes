import { enqueueBulkCrawlReferralLinks } from '@queues/crawl-referral-links/enqueues';
interface Dependencies { enqueue?: typeof enqueueBulkCrawlReferralLinks }
type Options = Dependencies & { urlId?: string };
export async function adopt(options: Options) { await options.enqueue!([], undefined); }
export function drop(options: Options) { options.enqueue!([], undefined); }
// These contracts deliberately exceed the bounded source projection.
export async function overlap(options: Dependencies & { enqueue: typeof enqueueBulkCrawlReferralLinks }) { await options.enqueue!([], undefined); }
export async function missing(options: {} & { urlId?: string }) { void options; }
export async function opaque(options: Dependencies & Unknown) { await options.enqueue!([], undefined); }
export async function union(options: Dependencies & ({ a: string } | { b: string })) { await options.enqueue!([], undefined); }
export async function indexed(options: Dependencies & { [key: string]: unknown }) { await options.enqueue!([], undefined); }
interface Base { a?: string }
interface Extended extends Base { b?: string }
export async function extended(options: Dependencies & Extended) { await options.enqueue!([], undefined); }
type Generic<T> = { value: T };
export async function generic(options: Dependencies & Generic<string>) { await options.enqueue!([], undefined); }
declare const key: unique symbol;
export async function computed(options: Dependencies & { [key]: string }) { await options.enqueue!([], undefined); }
// Cyclic source contracts remain unproven even if a type checker rejects them too.
type Cycle = Cycle & {};
export async function cyclic(options: Dependencies & Cycle) { await options.enqueue!([], undefined); }
