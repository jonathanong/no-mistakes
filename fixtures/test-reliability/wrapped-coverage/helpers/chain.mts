import { extWrap } from "external-test-lib";
import { arrowExpr, fnExpr } from "./barrel.mts";
import * as leafNs from "./leaf.mts";

export { arrowExpr as reexportedArrow };

export function delegated(t: unknown) {
  return arrowExpr(leafNs.arrowBlock(fnExpr(t, true)));
}

export function delegatedWithExtraArgs(t: unknown, extra: unknown[]) {
  return arrowExpr(...extra, t);
}

export function callsExternal(t: unknown) {
  return extWrap(t);
}

export const notAWrapper = 42;

export function cycleA(t: unknown): unknown {
  return cycleB(t);
}

export function cycleB(t: unknown): unknown {
  return cycleA(t);
}

export function bareReturn(t: unknown) {
  if (!t) {
    {
      return;
    }
  }
  return t;
}

export function mismatchParams(a: unknown, b: unknown, flag: boolean) {
  if (flag) {
    return a;
  }
  return b;
}
