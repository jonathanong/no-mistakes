import { expect as check } from "vitest";
import * as runner from "@jest/globals";
declare function work(): Promise<void>;
declare function release(): Promise<void>;

export async function callForwarding() {
  const promise = work();
  await release();
  await check.call(null, promise).rejects.toThrow();
}
export async function applyForwarding() {
  const promise = work();
  await release();
  await runner[`expect`]["apply"](null, [promise])["rejects"].toThrow();
}
export async function assertedForwarding() {
  const promise = work();
  await release();
  await (check as typeof check)?.[`call`](null, promise)[`rejects`].toThrow();
}
export async function nestedForwarding() {
  const promise = work();
  await release();
  await check.call.call(check, null, promise).rejects.toThrow();
}
export async function aliasedForwarding() {
  const promise = work();
  await release();
  const expectation = runner.expect;
  await expectation.apply(null, [promise]).rejects.toThrow();
}
