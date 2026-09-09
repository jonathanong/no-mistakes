import { setTimeout } from "../clock.mts";
import { setTimeout as wait } from "node:timers";

export function run() {
  // Unaliased local import spelled setTimeout must not match global or node:timers.
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
}
