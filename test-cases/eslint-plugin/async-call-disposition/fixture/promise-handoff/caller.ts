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

import { enqueueEmail as declaredEmail } from '@app/declared-jobs';
import { enqueueEmail as ambientEmail } from '@app/ambient-jobs';
import { enqueueEmail as synchronousEmail } from '@app/non-promise-jobs';
import { enqueueEmail as shadowedEmail } from '@app/shadowed-promise-jobs';
import { enqueueDropped, declaredSlot, ambientSlot, synchronousSlot, shadowedSlot, aliasedSlot, partialSlot, opaqueTyped, opaqueMember, untypedForward } from './consumers';
const declared = (entries: unknown, options: unknown) => declaredEmail(entries, options);
declaredSlot({ x: declared });
const ambient: typeof ambientEmail = (entries, options) => ambientEmail(entries, options);
ambientSlot({ x: ambient });
const synchronous: typeof synchronousEmail = (entries, options) => synchronousEmail(entries, options);
synchronousSlot({ x: synchronous });
const shadowed: typeof shadowedEmail = (entries, options) => shadowedEmail(entries, options);
shadowedSlot({ x: shadowed });
const aliasJob: typeof enqueueEmail = (entries, options) => enqueueEmail(entries, options);
aliasedSlot({ x: aliasJob });
const partialJob = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
partialSlot({ x: partialJob });
const droppedJob: typeof enqueueEmail = (entries, options) => enqueueEmail(entries, options);
enqueueDropped({ x: droppedJob, urlId: 'drop' });
const unknownObject = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
opaqueTyped({ x: unknownObject });
const unknownMember = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
opaqueMember({ x: unknownMember });
const untypedConsumer = (entries: unknown, options: unknown) => enqueueEmail(entries, options);
untypedForward({ x: untypedConsumer });
