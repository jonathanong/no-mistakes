import { expect } from "vitest";
async function reads() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await (expect(pending).rejects as any).toThrow();
}
