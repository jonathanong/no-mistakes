import { enqueueEmail } from "@app/jobs";

// The selected property is copied inside a handler that this module only returns.
// Adoption is not visible to the cross-file copy proof; a typed target slot is.
export function enqueueReferralLinkCrawlsForUrlId(dependencies: {
  x: typeof enqueueEmail;
  urlId: string;
}) {
  return prepareCrawl({ ...dependencies, urlId });
}

function prepareCrawl(options: { x: typeof enqueueEmail; urlId: string }) {
  const handler = () => {
    const crawl = options.x ?? enqueueEmail;
    return { crawl, urlId: options.urlId };
  };
  return handler;
}

export function startAny(options: any) {
  return options;
}

export function structural(callback: () => Promise<unknown>) {
  callback();
}

export function takeString(options: { note: string }) {
  return options.note;
}
