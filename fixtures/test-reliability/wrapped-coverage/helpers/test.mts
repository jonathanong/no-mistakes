import { test as base } from "@playwright/test";
import {
  bareReturn,
  callsExternal,
  cycleA,
  delegated,
  delegatedWithExtraArgs,
  mismatchParams,
  notAWrapper,
  reexportedArrow,
} from "./chain.mts";

function localDelegated(t: typeof base) {
  return reexportedArrow(t);
}

const extra: unknown[] = [];

export const good = localDelegated(delegated(base));
export const spreadCall = delegated(...extra, base);
export const noArgOrigin = delegated(123);
export const deepMember = (() => ({ wrap: delegated }))().wrap(base);
export const badCycle = cycleA(base);
export const badBare = bareReturn(base);
export const badMismatch = mismatchParams(base, base, true);
export const badExternal = callsExternal(base);
export const badSpread = delegatedWithExtraArgs(base, []);
export const unrelated = { notAWrapper };
