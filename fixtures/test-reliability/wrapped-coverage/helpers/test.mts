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

export const good = reexportedArrow(delegated(base));
export const badCycle = cycleA(base);
export const badBare = bareReturn(base);
export const badMismatch = mismatchParams(base, base, true);
export const badExternal = callsExternal(base);
export const badSpread = delegatedWithExtraArgs(base, []);
export const unrelated = { notAWrapper };
