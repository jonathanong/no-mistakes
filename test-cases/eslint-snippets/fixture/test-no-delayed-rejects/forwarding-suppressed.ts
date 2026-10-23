import { expect } from "vitest";
export async function suppression() {
  const promise = work();
  await release();
  // no-mistakes-disable-next-line test-no-delayed-rejects -- protocol fixture
  await expect.call(null, promise).rejects.toThrow();
  await expect.apply(null, [promise]).rejects.toThrow(); // no-mistakes-disable-line test-no-delayed-rejects -- protocol fixture
}
