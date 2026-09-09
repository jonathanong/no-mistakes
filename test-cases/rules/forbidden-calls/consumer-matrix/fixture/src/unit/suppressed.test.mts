import { setTimeout as wait } from "node:timers";

export function suppressedTimer() {
  // no-mistakes-disable-next-line forbidden-calls: fixture proves findings-only suppression
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
}
