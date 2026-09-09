import { setTimeout as wait } from "node:timers";

export function sleep() {
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
}

// Typed Playwright-style callback parameter: `terminal: waitForTimeout`
// must report this direct call without a locally initialized receiver.
test("timeout", async ({
  page,
}: {
  page: { waitForTimeout: (ms: number) => void };
}) => {
  page.waitForTimeout(1);
});
