import { test as base } from "@playwright/test";
import { bareReturn, cycleA, delegated, mismatchParams, reexportedArrow } from "./chain.mts";

export const good = reexportedArrow(delegated(base));
export const badCycle = cycleA(base);
export const badBare = bareReturn(base);
export const badMismatch = mismatchParams(base, base, true);
