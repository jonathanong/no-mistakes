import { expect } from "vitest";

declare function release(): Promise<void>;
declare function loadExpected(): Promise<unknown>;
declare function startOperation(): Promise<void>;
declare function dangerous(): void;

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

export async function assertedArgument() {
  const update = startOperation();
  await release();
  await expect(update as Promise<void>).rejects.toThrow();
}

export async function satisfiesArgument() {
  const update = startOperation();
  await release();
  await expect(update satisfies Promise<void>).rejects.toThrow();
}

export async function nonNullArgument() {
  const update = startOperation();
  await release();
  await expect(update!).rejects.toThrow();
}

export async function awaitInMatcherArgument() {
  const update = startOperation();
  await expect(update).rejects.toEqual(await loadExpected());
}

export async function conditionalCatchDoesNotDominate() {
  const update = startOperation();
  if (Math.random() > 0.5) void update.catch(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function absentCatchHandler() {
  const update = startOperation();
  void update.catch(undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function rethrowingCatchCreatesUnhandledChild() {
  const update = startOperation();
  void update.catch((error: unknown) => {
    throw error;
  });
  await release();
  await expect(update).rejects.toThrow();
}

export async function finallyRunsAfterReturn() {
  const update = startOperation();
  try {
    await release();
    return;
  } finally {
    await expect(update).rejects.toThrow();
  }
}

export async function throwingVoidExpressionIsNotSafe() {
  const update = startOperation();
  void update.catch(() => void dangerous());
  await release();
  await expect(update).rejects.toThrow();
}

export async function throwReachesCatchMatcher() {
  const update = startOperation();
  try {
    await release();
    throw new Error("expected");
  } catch {
    await expect(update).rejects.toThrow();
  }
}
