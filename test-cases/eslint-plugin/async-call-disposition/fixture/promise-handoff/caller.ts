import { enqueueEmail } from "@app/jobs";
import { enqueueReferralLinkCrawlsForUrlId, startAny, structural, takeString } from "./consumers";

const enqueueAndRecord: typeof enqueueEmail = (entries: unknown, options: unknown) => {
  void entries;
  return enqueueEmail(entries, options);
};
enqueueReferralLinkCrawlsForUrlId({ x: enqueueAndRecord, urlId: "1" });

const expressionBody: typeof enqueueEmail = (entries: unknown, options: unknown) =>
  enqueueEmail(entries, options);
enqueueReferralLinkCrawlsForUrlId({ x: expressionBody, urlId: "2" });

const inferred = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
enqueueReferralLinkCrawlsForUrlId({ x: inferred, urlId: "3" });

const anySlot: typeof enqueueEmail = (entries: unknown, options: unknown) => {
  return enqueueEmail(entries, options);
};
startAny({ x: anySlot });

const bare: typeof enqueueEmail = (entries: unknown, options: unknown) => {
  enqueueEmail(entries, options);
};
enqueueReferralLinkCrawlsForUrlId({ x: bare, urlId: "4" });

const structuralSlot: typeof enqueueEmail = (entries: unknown, options: unknown) => {
  return enqueueEmail(entries, options);
};
structural(structuralSlot);

const stringSlot = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
takeString({ note: stringSlot });

const asserted: typeof enqueueEmail = (entries: unknown, options: unknown) => {
  return enqueueEmail(entries, options);
};
structural(asserted as typeof enqueueEmail);
