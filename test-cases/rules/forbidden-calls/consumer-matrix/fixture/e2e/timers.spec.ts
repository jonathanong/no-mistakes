import { setTimeout as wait } from "node:timers";
import * as timers from "node:timers";
import { setTimeout as waitP } from "node:timers/promises";
import * as timersP from "node:timers/promises";

export function sleep() {
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
  timers.setTimeout(() => {}, 1);
  void waitP(1);
  timersP.setTimeout(1);
}

const actor = {
  waitForTimeout(_ms: number) {},
  run() {
    this.waitForTimeout(1);
  },
};

function factory(): { waitForTimeout: (ms: number) => void } {
  return { waitForTimeout() {} };
}

// Typed Playwright-style callback parameter and renamed receiver.
test("timeout", async ({
  page,
  browser,
}: {
  page: { waitForTimeout: (ms: number) => void };
  browser: { waitForTimeout: (ms: number) => void };
}) => {
  page.waitForTimeout(1);
  browser.waitForTimeout(1);
  // Static computed access is a distinct call site from page.waitForTimeout().
  page["waitForTimeout"](1);
  factory().waitForTimeout(1);
  actor.run();
  // Dynamic computed access stays ignored with unknownCalls: ignore.
  const method = "waitForTimeout";
  page[method](1);
});

