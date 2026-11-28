import { expect } from "vitest";
async function reads() {
  const pending = Promise.reject(new Error("x"));
  (pending.catch as typeof pending.catch)(() => {});
  await Promise.resolve();
  await expect(pending).rejects.toThrow();
}
