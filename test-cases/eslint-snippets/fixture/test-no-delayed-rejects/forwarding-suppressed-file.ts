// no-mistakes-disable-file test-no-delayed-rejects -- protocol fixture
import { expect } from "vitest";
export async function suppression() {
  const promise = work();
  await release();
  await expect.call(null, promise).rejects.toThrow();
}
