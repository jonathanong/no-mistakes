import { setTimeout as importedTimeout } from "node:timers";
import * as timers from "node:timers";
import { reexportedTarget } from "./barrel.mts";
import { repositoryTarget } from "./targets.mts";

export function selectorCalls() {
  importedTimeout(() => {}, 1);
  timers.setTimeout(() => {}, 1);
  repositoryTarget();
  reexportedTarget();
  Date();
  new Date();
}

export function shadowed(setTimeout: () => void) {
  setTimeout();
}

export function unknownCall(runner: Record<string, () => void>, name: string) {
  runner[name]();
}

export function timeoutCall(page: { waitForTimeout: (ms: number) => void }) {
  page.waitForTimeout(1);
}
