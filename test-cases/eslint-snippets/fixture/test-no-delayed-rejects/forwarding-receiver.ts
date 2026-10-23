import { expect } from "vitest";
// This matcher-shaped result never observes the supplied promise.
function other(_promise: unknown) { return { rejects: { toThrow() {} } }; }
export async function wrongNestedReceiver() {
  const promise = work();
  expect.call.call(other, null, promise).rejects.toThrow();
  await release();
  await expect(promise).rejects.toThrow();
}
export async function wrongApplyReceiver() {
  const promise = work();
  expect.apply.call(other, null, [promise]).rejects.toThrow();
  await release();
  await expect(promise).rejects.toThrow();
}
export async function detachedIntrinsic() {
  const promise = work();
  const invoke = expect.call;
  invoke.call(other, null, promise).rejects.toThrow();
  await release();
  await expect(promise).rejects.toThrow();
}
export async function provenNestedReceiver() {
  const promise = work();
  expect.call.call(expect, null, promise).rejects.toThrow();
  await release();
  await expect(promise).rejects.toThrow();
}
