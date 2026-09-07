import { expect } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;
declare const dynamicProperty: string;
declare const service: { start(handler: unknown): Promise<void> };
declare function getService(): { start(handler: unknown): Promise<void> };

export async function noInterveningAwait() {
  const update = startOperation();
  await expect(update).rejects.toThrow();
}

export async function immediateCatchCapture() {
  const update = startOperation();
  const rejection = update.catch((error: unknown) => error);
  await release();
  await expect(rejection).resolves.toMatchObject({ status: 403 });
}

export async function immediateCatchObserver() {
  const update = startOperation();
  void update.catch(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function immediateBlockCatchObserver() {
  const update = startOperation();
  void update.catch(() => {
    return undefined;
  });
  await release();
  await expect(update).rejects.toThrow();
}

export async function immediateThenObserver() {
  const update = startOperation();
  void update.then(undefined, () => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function terminalCatchObservesPromiseChain() {
  const update = startOperation();
  void update.then(() => undefined).catch(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function safeContinuationAfterSafeCatch() {
  const update = startOperation();
  void update.catch(() => undefined).then(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function safeMultiStageContinuation() {
  const update = startOperation();
  void update
    .catch(() => undefined)
    .finally(() => undefined)
    .then(() => {
      throw new Error("recovered below");
    })
    .then(
      () => undefined,
      () => undefined,
    )
    .catch(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function inertVoidFulfillmentHandler() {
  const update = startOperation();
  void update.then(void 0, () => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerAttachedInsideAwait() {
  const update = startOperation();
  await update.catch(() => undefined);
  await expect(update).rejects.toThrow();
}

export async function observerDominatesNestedAwait(flag: boolean) {
  const update = startOperation();
  void update.catch(() => undefined);
  if (flag) await release();
  await expect(update).rejects.toThrow();
}

export async function observerInUnconditionalNestedBlock() {
  const update = startOperation();
  {
    void update.catch(() => undefined);
  }
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInFinallyDominatesLaterAwait() {
  const update = startOperation();
  try {
    Math.random();
  } finally {
    void update.catch(() => undefined);
  }
  await release();
  await expect(update).rejects.toThrow();
}

export async function exitingTryDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    return;
  } finally {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function returningTryWithCatchDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    return;
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function nestedReturningTryDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    try {
      return;
    } finally {
      Math.random();
    }
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function terminatingBranchDoesNotReachAssertion(skip: boolean) {
  const update = startOperation();
  if (skip) {
    await release();
    return;
  }
  await expect(update).rejects.toThrow();
}

export async function returnAwaitDoesNotReachAssertion(skip: boolean) {
  const update = startOperation();
  if (skip) return await release();
  await expect(update).rejects.toThrow();
}

export async function throwingBranchDoesNotReachAssertion(skip: boolean) {
  const update = startOperation();
  if (skip) {
    await release();
    throw new Error("expected");
  }
  await expect(update).rejects.toThrow();
}

export async function breakingBranchDoesNotReachAssertion(skip: boolean) {
  const update = startOperation();
  while (true) {
    if (skip) {
      await release();
      break;
    }
    await expect(update).rejects.toThrow();
    break;
  }
}

export async function nestedBlockBreakDoesNotReachAssertion(skip: boolean) {
  const update = startOperation();
  while (true) {
    if (skip) {
      await release();
      {
        break;
      }
    }
    await expect(update).rejects.toThrow();
    break;
  }
}

export async function earlierMatcherObservesBeforeLaterSuspension() {
  const update = startOperation();
  await expect(update).rejects.toThrow();
  await release();
  await expect(update).rejects.toMatchObject({ status: 403 });
}

export async function initializerSuspendsBeforePromiseCreation() {
  const update = startOperation(await release());
  await expect(update).rejects.toThrow();
}

export async function caughtThrowReturningFromFunctionDoesNotReachMatcher() {
  const update = startOperation();
  try {
    await release();
    throw new Error("caught");
  } catch {
    return;
  }
  await expect(update).rejects.toThrow();
}

export async function methodInitializerSuspendsBeforePromiseCreation() {
  const update = service.start(await release());
  await expect(update).rejects.toThrow();
}

export async function factoryMethodInitializerSuspendsBeforePromiseCreation() {
  const update = getService().start(await release());
  await expect(update).rejects.toThrow();
}

export async function mutuallyExclusiveBranches(skip: boolean) {
  const update = startOperation();
  if (skip) {
    await release();
  } else {
    await expect(update).rejects.toThrow();
  }
}

export async function mutuallyExclusiveSwitchCases(kind: "wait" | "assert") {
  const update = startOperation();
  switch (kind) {
    case "wait":
      await release();
      break;
    case "assert":
      await expect(update).rejects.toThrow();
      break;
  }
}

export async function matcherBeforeLaterLoopAwait() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    await expect(update).rejects.toThrow();
    await release();
  }
}

export async function matcherInLoopIfTestDominatesLaterAwait() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    if (await expect(update).rejects.toThrow()) continue;
    await release();
  }
}

export async function matcherInLoopConditionalTestDominatesLaterAwait() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    const result = (await expect(update).rejects.toThrow()) ? 1 : 2;
    if (result === 1) continue;
    await release();
  }
}

export async function matcherInLoopLogicalLeftDominatesLaterAwait() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    (await expect(update).rejects.toThrow()) && index.toString();
    await release();
  }
}

export async function nestedCallbackAwait() {
  const update = startOperation();
  void (async () => {
    await release();
  })();
  await expect(update).rejects.toThrow();
}

export async function shadowedBinding() {
  const update = startOperation();
  await release();
  {
    const update = startOperation();
    await expect(update).rejects.toThrow();
  }
}

export async function resolvesInstead() {
  const update = startOperation();
  await release();
  await expect(update).resolves.toBeUndefined();
}

export async function inlineExpectation() {
  await release();
  await expect(startOperation()).rejects.toThrow();
}

export async function unresolvedBindingIsUnsupported() {
  await release();
  await expect(unresolved).rejects.toThrow();
}

export async function dynamicRejectsPropertyIsUnsupported() {
  const update = startOperation();
  await release();
  await expect(update)[dynamicProperty].toThrow();
}

export async function expectWithMultipleArgumentsIsUnsupported() {
  const update = startOperation();
  await release();
  await expect(update, "extra context").rejects.toThrow();
}

export async function aggregateObservesBeforeAwait() {
  const update = startOperation();
  await Promise.all([expect(update).rejects.toThrow()]);
}

export async function storedMatcherPromiseIsOutsideThisRule() {
  const update = startOperation();
  const matcher = expect(update).rejects.toThrow();
  await release();
  await matcher;
}

export async function mutableBinding() {
  let update = startOperation();
  await release();
  update = startOperation();
  await expect(update).rejects.toThrow();
}

export async function alias() {
  const update = startOperation();
  const alias = update;
  await release();
  await expect(alias).rejects.toThrow();
}

export async function crossFunctionFlow() {
  const update = startOperation();
  await release();
  await (async () => expect(update).rejects.toThrow())();
}

export async function shadowedExpect() {
  const update = startOperation();
  await release();
  const expect = (_value: unknown) => ({ rejects: { toThrow: () => undefined } });
  await expect(update).rejects.toThrow();
}
