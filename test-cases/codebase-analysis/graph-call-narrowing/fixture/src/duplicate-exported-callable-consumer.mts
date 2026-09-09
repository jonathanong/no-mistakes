import { run, target } from "./duplicate-exported-callable.mts";

export function callRun() {
  run();
}

export function callTarget() {
  target();
}
