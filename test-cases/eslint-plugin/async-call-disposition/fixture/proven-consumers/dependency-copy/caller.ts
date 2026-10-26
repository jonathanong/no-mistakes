import { enqueueEmail } from "@app/jobs";
import {
  start,
  startAlias,
  startAliasDropped,
  startAliasMember,
  startAliasOptional,
  startBuilt,
  startCallObject,
  startChained,
  startDivergent,
  startDropped,
  startDynamic,
  startEither,
  startEscape,
  startFloat,
  startForgot,
  startInline,
  startLocal,
  startMixed,
  startMutated,
  startOnlyRead,
  startOptional,
  startOptionalCall,
  startParamAdoptThenReturn,
  startProto,
  startRead,
  startReturnedClosure,
  startReturnedObjectClosure,
  startRhs,
  startSafeSibling,
  startStoredClosure,
  startSibling,
  startTwice,
  startUnawaitedCall,
  startUnknown,
  startOptionalAbsent,
  startOptionalUnsafe,
  startParamReturnFirst,
  startReturnFirst,
  startUnread,
  startVoid,
} from "./build";

// The callback returns the target promise. build.ts copies it through ?? onto a
// dependencies object, and run.ts awaits that property. Inlining the copy into
// the awaited call would hide this three-file gap.
function record(entries: unknown) {
  void entries;
}

const enqueueAndRecord = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await start({ enqueueBulkCrawlReferralLinks: enqueueAndRecord });

const built = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startBuilt({ enqueueBulkCrawlReferralLinks: built });

const inlined = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startInline({ enqueueBulkCrawlReferralLinks: inlined });

const dropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDropped({ enqueueBulkCrawlReferralLinks: dropped });

const unawaitedCall = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startUnawaitedCall({ enqueueBulkCrawlReferralLinks: unawaitedCall });

const mutated = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startMutated({ enqueueBulkCrawlReferralLinks: mutated });

const rhs = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startRhs({ enqueueBulkCrawlReferralLinks: rhs });

const unknown = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startUnknown({ enqueueBulkCrawlReferralLinks: unknown });

const escaped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startEscape({ enqueueBulkCrawlReferralLinks: escaped });

const forgot = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startForgot({ enqueueBulkCrawlReferralLinks: forgot });

const either = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startEither({ enqueueBulkCrawlReferralLinks: either }, true);

const aliased = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startAlias({ enqueueBulkCrawlReferralLinks: aliased });

const localAwait = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startLocal({ enqueueBulkCrawlReferralLinks: localAwait });

const readSibling = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startRead({ enqueueBulkCrawlReferralLinks: readSibling });

const safeSibling = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startSafeSibling({ enqueueBulkCrawlReferralLinks: safeSibling });

const aliasDropped = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startAliasDropped({ enqueueBulkCrawlReferralLinks: aliasDropped });

const aliasOptional = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startAliasOptional({ enqueueBulkCrawlReferralLinks: aliasOptional });

const aliasMember = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startAliasMember({ enqueueBulkCrawlReferralLinks: aliasMember });

const chained = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startChained({ enqueueBulkCrawlReferralLinks: chained });

const sibling = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startSibling({ enqueueBulkCrawlReferralLinks: sibling });

const floated = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startFloat({ enqueueBulkCrawlReferralLinks: floated });

const mixed = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startMixed({ enqueueBulkCrawlReferralLinks: mixed });

const optional = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptional({ enqueueBulkCrawlReferralLinks: optional });

const proto = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startProto({ enqueueBulkCrawlReferralLinks: proto });

const dynamic = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDynamic({ enqueueBulkCrawlReferralLinks: dynamic });

const calledObject = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startCallObject({ enqueueBulkCrawlReferralLinks: calledObject });

const optionalCall = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalCall({ enqueueBulkCrawlReferralLinks: optionalCall });

const unread = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startUnread({ enqueueBulkCrawlReferralLinks: unread });

const onlyRead = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOnlyRead({ enqueueBulkCrawlReferralLinks: onlyRead });

const twice = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
startTwice({ enqueueBulkCrawlReferralLinks: twice }, true);

const ignored = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startVoid({ enqueueBulkCrawlReferralLinks: ignored });

const returnedFirst = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startReturnFirst({ enqueueBulkCrawlReferralLinks: returnedFirst }, true);

const optionalAbsent = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalAbsent({ enqueueBulkCrawlReferralLinks: optionalAbsent });

const optionalUnsafe = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startOptionalUnsafe({ enqueueBulkCrawlReferralLinks: optionalUnsafe });

const divergent = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startDivergent({ enqueueBulkCrawlReferralLinks: divergent }, true);

const paramReturnFirst = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startParamReturnFirst({ enqueueBulkCrawlReferralLinks: paramReturnFirst }, true);

const adoptedThenReturned = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startParamAdoptThenReturn({ enqueueBulkCrawlReferralLinks: adoptedThenReturned });

const returnedClosure = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startReturnedClosure({ enqueueBulkCrawlReferralLinks: returnedClosure });

const returnedObjectClosure = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startReturnedObjectClosure({ enqueueBulkCrawlReferralLinks: returnedObjectClosure });

const storedClosure = (entries: unknown, options: unknown) => {
  record(entries);
  return enqueueEmail(entries, options);
};
await startStoredClosure({ enqueueBulkCrawlReferralLinks: storedClosure });
