import * as pw from "@playwright/test";
import test from "@playwright/test";
declare function start(): Promise<void>;
declare function release(): Promise<void>;
// Playwright assertions deliberately stay outside the delayed-rejects runner contract.
async function namespace() {
  const operation = start();
  await release();
  await pw.expect(operation).rejects.toThrow();
}
async function defaultRunner() {
  const operation = start();
  await release();
  await test.expect(operation).rejects.toThrow();
}
