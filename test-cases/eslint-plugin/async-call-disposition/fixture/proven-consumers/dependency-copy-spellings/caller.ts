import { enqueueEmail } from "@app/jobs";
import {
  startConditional,
  startDefaulted,
  startDefaultedDropped,
  startComputedKey,
  startDestructured,
  startDestructuredDropped,
  startLet,
  startNested,
  startOptionalBare,
  startOptionalSource,
  startOptionalSourceDropped,
  startOr,
  startOrAsync,
  startOrAsyncDropped,
  startOrDropped,
  startOrRight,
  startRenamed,
  startRest,
  startSpread,
  startSpreadDropped,
  startSpreadOverwrite,
} from "./builders";

function record(entries: unknown) {
  void entries;
}

const orCallback = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOr({ enqueueBulkCrawlReferralLinks: orCallback });

const orDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOrDropped({ enqueueBulkCrawlReferralLinks: orDropped });

const orAsync = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOrAsync({ enqueueBulkCrawlReferralLinks: orAsync });

const orAsyncDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOrAsyncDropped({ enqueueBulkCrawlReferralLinks: orAsyncDropped });

const orRight = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOrRight({ enqueueBulkCrawlReferralLinks: orRight });

const defaulted = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDefaulted({ enqueueBulkCrawlReferralLinks: defaulted });

const defaultedDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDefaultedDropped({ enqueueBulkCrawlReferralLinks: defaultedDropped });

const destructured = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDestructured({ enqueueBulkCrawlReferralLinks: destructured });

const destructuredDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDestructuredDropped({ enqueueBulkCrawlReferralLinks: destructuredDropped });

const rest = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startRest({ enqueueBulkCrawlReferralLinks: rest });

const spread = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startSpread({ enqueueBulkCrawlReferralLinks: spread });

const spreadDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startSpreadDropped({ enqueueBulkCrawlReferralLinks: spreadDropped });

const spreadOverwrite = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startSpreadOverwrite({ enqueueBulkCrawlReferralLinks: spreadOverwrite });

const optionalSource = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalSource({ enqueueBulkCrawlReferralLinks: optionalSource });

const optionalSourceDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalSourceDropped({ enqueueBulkCrawlReferralLinks: optionalSourceDropped });

const optionalBare = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalBare({ enqueueBulkCrawlReferralLinks: optionalBare });

const conditional = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startConditional({ enqueueBulkCrawlReferralLinks: conditional }, true);

const letBinding = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startLet({ enqueueBulkCrawlReferralLinks: letBinding });

const renamed = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startRenamed({ enqueueBulkCrawlReferralLinks: renamed });

const nested = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startNested({ enqueueBulkCrawlReferralLinks: nested });

const computedKey = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startComputedKey({ enqueueBulkCrawlReferralLinks: computedKey });
