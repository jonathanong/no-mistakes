import { expect } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function pollCase() {
  const update = startOperation();
  await release();
  await expect.poll(() => update).rejects.toThrow();
  await expect.call(update).rejects.toThrow();
  await expect.apply(update).rejects.toThrow();
}
