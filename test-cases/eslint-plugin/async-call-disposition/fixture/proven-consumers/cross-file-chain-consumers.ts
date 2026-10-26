import { enqueueByHostname } from "./cross-file-chain-final";
// This saved shape mirrors a defaulted dependency wrapper, selected-property
// forwarding through a lexical batch handler, and an imported awaiting helper.
export async function enqueueReferralLinkCrawlsForUrlId(urlId: string, dependencies: any = {}) {
  await dispatchReferralLinkCrawls({ ...dependencies, urlId });
}
async function dispatchReferralLinkCrawls(options: any = {}) {
  const urlId = options.urlId;
  await options.saveProgress?.();
  await runBatches({ handler: async rows => {
    await enqueueByHostname(rows, {
      enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? fallback,
      computeHostnameRateLimitMs: options.computeHostnameRateLimitMs ?? fallback,
    });
  } });
  return urlId;
}
export async function unknownForwarder(dependencies: any) {
  await unknown({ enqueueBulkCrawlReferralLinks: dependencies.enqueueBulkCrawlReferralLinks });
}
export async function overwritten(dependencies: any) {
  await dispatchReferralLinkCrawls({ ...dependencies, enqueueBulkCrawlReferralLinks: fallback });
}
export async function cycle(dependencies: any) { await cycleOther({ ...dependencies }); }
async function cycleOther(dependencies: any) { await cycle({ ...dependencies }); }
export async function escaped(dependencies: any) {
  leak(dependencies);
  await dispatchReferralLinkCrawls({ ...dependencies });
}
export async function stored(dependencies: any) {
  const handler = async () => await dependencies.enqueueBulkCrawlReferralLinks();
  leak(handler);
}
async function runBatches(dependencies: any) { await dependencies.handler([]); }
async function fallback() {}
declare function unknown(value: any): Promise<void>;
declare function leak(value: any): void;
export async function aliasedContainer(dependencies: any) {
  const copy = dependencies;
  await enqueueByHostname([], { ...dependencies, computeHostnameRateLimitMs: () => { copy.enqueueBulkCrawlReferralLinks(); } });
}
export async function aliasedProperty(dependencies: any) {
  const callback = dependencies.enqueueBulkCrawlReferralLinks;
  await enqueueByHostname([], { ...dependencies, computeHostnameRateLimitMs: () => { callback(); } });
}
