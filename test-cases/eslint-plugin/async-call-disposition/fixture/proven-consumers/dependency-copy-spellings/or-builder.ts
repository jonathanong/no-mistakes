async function defaultImpl(_entries: unknown, _options: unknown) {
  return undefined;
}

// The || copy lives in this module. builders.ts awaits the returned object.
export async function buildOr(options: any) {
  return {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks || defaultImpl,
  };
}
