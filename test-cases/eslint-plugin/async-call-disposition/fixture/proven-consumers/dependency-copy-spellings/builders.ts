import { buildOr } from "./or-builder";
import { crawl } from "./run";

async function defaultImpl(_entries: unknown, _options: unknown) {
  return undefined;
}

export async function startOr(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks || defaultImpl,
  };
  await crawl(dependencies);
}

export async function startOrDropped(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks || defaultImpl,
  };
  crawl(dependencies);
}

export async function startOrAsync(options: any) {
  const dependencies = await buildOr(options);
  await crawl(dependencies);
}

export async function startOrAsyncDropped(options: any) {
  const dependencies = await buildOr(options);
  crawl(dependencies);
}

// The callback is the right operand, so || does not copy it.
export async function startOrRight(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: defaultImpl || options.enqueueBulkCrawlReferralLinks,
  };
  await crawl(dependencies);
}

export async function startDefaulted(options: any) {
  const { enqueueBulkCrawlReferralLinks = defaultImpl } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  await crawl(dependencies);
}

export async function startDefaultedDropped(options: any) {
  const { enqueueBulkCrawlReferralLinks = defaultImpl } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  crawl(dependencies);
}

export async function startDestructured(options: any) {
  const { "label": label } = options;
  void label;
  const { enqueueBulkCrawlReferralLinks } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  await crawl(dependencies);
}

export async function startDestructuredDropped(options: any) {
  const { enqueueBulkCrawlReferralLinks } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  crawl(dependencies);
}

export async function startLet(options: any) {
  let { enqueueBulkCrawlReferralLinks } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  await crawl(dependencies);
}

export async function startRenamed(options: any) {
  const { enqueueBulkCrawlReferralLinks: renamed } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks: renamed };
  await crawl(dependencies);
}

export async function startNested(options: any) {
  const { label: { extra } } = options;
  void extra;
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await crawl(dependencies);
}

export async function startComputedKey(options: any) {
  const key: string = "enqueueBulkCrawlReferralLinks";
  const { [key]: enqueueBulkCrawlReferralLinks } = options;
  const dependencies = { enqueueBulkCrawlReferralLinks };
  await crawl(dependencies);
}

// Rest can carry the callback without naming it.
export async function startRest(options: any) {
  const { ...dependencies } = options;
  await crawl(dependencies);
}

// The spread overwrites the earlier default, so the source property remains.
export async function startSpread(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: defaultImpl,
    ...options,
    "label": "crawl",
  };
  await crawl(dependencies);
}

export async function startSpreadDropped(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: defaultImpl,
    ...options,
  };
  crawl(dependencies);
}

// A key after the spread replaces the copied callback.
export async function startSpreadOverwrite(options: any) {
  const dependencies = {
    ...options,
    enqueueBulkCrawlReferralLinks: defaultImpl,
  };
  await crawl(dependencies);
}

export async function startOptionalSource(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options?.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await crawl(dependencies);
}

export async function startOptionalSourceDropped(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options?.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  crawl(dependencies);
}

// Optional member alone is not one of the copy operators above.
export async function startOptionalBare(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options?.enqueueBulkCrawlReferralLinks,
  };
  await crawl(dependencies);
}

// The condition does not have to be unconditional. The awaited call is the adoption.
export async function startConditional(options: any, flag: boolean) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  if (flag) await crawl(dependencies);
}
