import { expect } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function oneAwait() {
  const update = startOperation();
  await release();
  await expect(update).rejects.toMatchObject({ status: 403 });
}

export async function multipleAwaits() {
  const update = startOperation();
  await release();
  await release();
  await expect(update).rejects.toThrow();
}

export async function branch() {
  const update = startOperation();
  if (Math.random() > 0.5) await release();
  await expect(update).rejects.toThrow();
}

export async function tryFinally() {
  const update = startOperation();
  try {
    await release();
  } finally {
    await release();
  }
  await expect(update).rejects.toThrow();
}

export async function authorizationRace() {
  const update = startOperation();
  await release();
  await release();
  await expect(update).rejects.toMatchObject({ status: 403 });
}

export async function computedRejects() {
  const update = startOperation();
  await release();
  await expect(update)["rejects"].toThrow();
}

export async function softExpectation() {
  const update = startOperation();
  await release();
  await expect.soft(update).rejects.toThrow();
}

export async function forAwaitSuspends(items: AsyncIterable<unknown>) {
  const update = startOperation();
  for await (const _item of items) {
    // Iteration can let update reject before the matcher attaches.
  }
  await expect(update).rejects.toThrow();
}
