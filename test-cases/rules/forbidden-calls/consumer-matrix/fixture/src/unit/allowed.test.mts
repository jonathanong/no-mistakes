import { setTimeout as wait } from "node:timers";

declare const test: { setTimeout: (ms: number) => void };

wait(() => {}, 1);

export function run(setTimeout: () => void) {
  setTimeout();
  test.setTimeout(30_000);
}
