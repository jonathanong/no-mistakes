import { createEnqueueFunction } from "../lib.mts";
import { viaExportFrom } from "../alias2.mts";
import { viaChain } from "../alias4.mts";
import { fromStar } from "../alias5.mts";
import { bg as starBg } from "../alias3.mts";
import * as qns from "../q.mts";
import { JOB_IMPORTED, NOT_A_STRING } from "../names.mts";
import { bg, other } from "../q.mts";

const LOCAL = "vLocal";
const suffix = "x";

export const eDirect = createEnqueueFunction(bg, "vDirect");
export const e2 = createEnqueueFunction(viaExportFrom, "vAlias2");
export const eChain = createEnqueueFunction(viaChain, "vChain");
export const eStar = createEnqueueFunction(fromStar, "vFromStar");
export const e3 = createEnqueueFunction(starBg, "vAlias3");
export const e7 = createEnqueueFunction(qns.bg, "vNamespace");
export const e6 = createEnqueueFunction(bg, JOB_IMPORTED);
export const eLocal = createEnqueueFunction(bg, LOCAL);
export const eTemplate = createEnqueueFunction(bg, `vTemplate`);
// A substitution is not a static job. It must not make `unproduced` look dead.
export const eDynamic = createEnqueueFunction(bg, `v${suffix}`);
export const eDead = createEnqueueFunction(other, "neverHandled");
// A non-string import does not become a job. An unknown binding is not a queue.
export const eBadJob = createEnqueueFunction(bg, NOT_A_STRING);
const dynName = suffix + "y";
export const eLost = createEnqueueFunction(notImported, dynName);
export const eNested = createEnqueueFunction(qns.bg.name, "nope");
