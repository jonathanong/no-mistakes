import { expect as check } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function kept() {
  const update = startOperation();
  await release();
  // no-mistakes-disable-next-line test-no-delayed-rejects -- retained probe
  await check(update).rejects.toThrow();
}

export async function reported() {
  const update = startOperation();
  await release();
  await check(update).rejects.toThrow();
}
