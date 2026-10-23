import { expect } from "vitest";
import { expect as unrelated } from "another-runner";
declare function work(): Promise<void>;
declare function release(): Promise<void>;
export async function malformedAndUnknown() {
  const promise = work();
  await release();
  // This passes promise as this, rather than as the expectation argument.
  await expect.call(promise).rejects.toThrow();
  await expect.apply(promise).rejects.toThrow();
  await expect.apply(null, unknownArgs).rejects.toThrow();
  await expect.apply(null, [...unknownArgs]).rejects.toThrow();
  await expect.call(null, ...unknownArgs).rejects.toThrow();
  await expect.apply.call(expect, null, unknownArgs).rejects.toThrow();
  const shape = { rejects: { toThrow() {} } };
  shape.rejects.toThrow();
  await expect.poll.call(null, () => promise).rejects.toThrow();
  await expect.poll.apply(null, [() => promise]).rejects.toThrow();
  const poll = expect.poll;
  const callback = () => promise;
  await poll.call(null, callback).rejects.toThrow();
  await unrelated.call(null, promise).rejects.toThrow();
}
export async function shadowed(expect: Function) {
  const promise = work();
  await release();
  await expect.call(null, promise).rejects.toThrow();
}
export async function earlyObserver() {
  const promise = work();
  // The observer attaches before release, so the later direct assertion is safe.
  const matcher = expect.apply(null, [promise]).rejects.toThrow();
  await release();
  await expect(promise).rejects.toThrow();
  await matcher;
}
export async function immediateObserver() {
  const promise = work();
  await expect.call(null, promise).rejects.toThrow();
}
