export async function floating(dependencies: any) { dependencies.enqueueBulkCrawlReferralLinks(); }
export async function escaped(dependencies: any) { await dependencies.enqueueBulkCrawlReferralLinks(); leak(dependencies); }
export async function replaced(dependencies: any) { dependencies = other; await dependencies.enqueueBulkCrawlReferralLinks(); }
export async function changed(dependencies: any) { dependencies.enqueueBulkCrawlReferralLinks = other; await dependencies.enqueueBulkCrawlReferralLinks(); }
export async function deferred(dependencies: any) { return async () => await dependencies.enqueueBulkCrawlReferralLinks(); }
export async function optional(dependencies: any) { await dependencies?.enqueueBulkCrawlReferralLinks(); }
export async function optionalCall(dependencies: any) { await dependencies.enqueueBulkCrawlReferralLinks?.(); }
export async function otherProperty(dependencies: any) { await dependencies.other(); }
export async function dynamic(dependencies: any) { await dependencies[key](); }
export async function defaultCall(dependencies: any, result = dependencies.enqueueBulkCrawlReferralLinks()) { await dependencies.enqueueBulkCrawlReferralLinks(); }
export async function rest(...dependencies: any[]) { await dependencies[0].enqueueBulkCrawlReferralLinks(); }
export async function destructured({ enqueueBulkCrawlReferralLinks }: any) { await enqueueBulkCrawlReferralLinks(); }
export async function noReferences(dependencies: any) { return; }
export async function evaluated(dependencies: any) { eval("dependencies = other"); await dependencies.enqueueBulkCrawlReferralLinks(); }
export async function argumentsRead(dependencies: any) { void arguments; await dependencies.enqueueBulkCrawlReferralLinks(); }
export default async function defaultConsumer(callback: any) { await callback(); }
declare const other: any, key: string;
declare function leak(value: any): void;
