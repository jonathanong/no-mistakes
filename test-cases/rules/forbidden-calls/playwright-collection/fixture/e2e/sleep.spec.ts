import { setTimeout as wait } from "node:timers";

export function sleep() {
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
}

export function timeoutCall(page: { waitForTimeout: (ms: number) => void }) {
  page.waitForTimeout(1);
}
