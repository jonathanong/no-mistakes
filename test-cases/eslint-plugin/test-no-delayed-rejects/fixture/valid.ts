import { expect } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;
declare const dynamicProperty: string;
declare const service: { start(handler: unknown): Promise<void> };
declare function getService(): { start(handler: unknown): Promise<void> };
declare function consume(...values: unknown[]): void;
declare function acquireLock(): AsyncDisposable;
declare const locks: Iterable<AsyncDisposable>;
declare const maybeCoordinator: undefined | { release(value: Promise<void>): Promise<void> };

export async function noInterveningAwait() {
  const update = startOperation();
  await expect(update).rejects.toThrow();
}

export async function directAwaitObservesRejection() {
  const update = startOperation();
  try {
    await update;
    return;
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function directlyAwaitedChainObservesRejection() {
  const update = startOperation();
  try {
    await update.then();
  } catch {
    Math.random();
  }
  await release();
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
  void update.catch(() => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function immediateBlockCatchObserver() {
  const update = startOperation();
  void update.catch(() => {
    return void 0;
  });
  await release();
  await expect(update).rejects.toThrow();
}

export async function immediateThenObserver() {
  const update = startOperation();
  void update.then(void 0, () => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function forAwaitIterableObserver(items: AsyncIterable<unknown>) {
  const update = startOperation();
  for await (const _item of [update.catch(() => void 0), items]) {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function parenthesizedOptionalMemberStillEvaluatesArguments() {
  const update = startOperation();
  try {
    await (maybeCoordinator?.release)(update.catch(() => void 0));
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function terminalCatchObservesPromiseChain() {
  const update = startOperation();
  void update.then(() => void 0).catch(() => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function safeContinuationAfterSafeCatch() {
  const update = startOperation();
  void update.catch(() => void 0).then(() => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function safeMultiStageContinuation() {
  const update = startOperation();
  void update
    .catch(() => void 0)
    .finally(() => void 0)
    .then(() => {
      throw new Error("recovered below");
    })
    .then(
      () => void 0,
      () => void 0,
    )
    .catch(() => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function inertVoidFulfillmentHandler() {
  const update = startOperation();
  void update.then(void 0, () => void 0);
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerAttachedInsideAwait() {
  const update = startOperation();
  await update.catch(() => void 0);
  await expect(update).rejects.toThrow();
}

export async function observerDominatesNestedAwait(flag: boolean) {
  const update = startOperation();
  void update.catch(() => void 0);
  if (flag) await release();
  await expect(update).rejects.toThrow();
}

export async function observerInIfTestDominatesAwait() {
  const update = startOperation();
  if (update.catch(() => void 0)) Math.random();
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInLogicalLeftDominatesAwait() {
  const update = startOperation();
  update.catch(() => void 0) && Math.random();
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInEarlierCallArgumentDominatesAwait() {
  const update = startOperation();
  consume(
    update.catch(() => void 0),
    await release(),
  );
  await expect(update).rejects.toThrow();
}

export async function observerInEarlierArrayElementDominatesAwait() {
  const update = startOperation();
  consume([update.catch(() => void 0), await release()]);
  await expect(update).rejects.toThrow();
}

export async function observerInEarlierSequenceEntryDominatesAwait() {
  const update = startOperation();
  consume((update.catch(() => void 0), await release()));
  await expect(update).rejects.toThrow();
}

export async function observerBeforeAwaitUsingScopeExit() {
  const update = startOperation();
  {
    await using _lock = acquireLock();
    void update.catch(() => void 0);
  }
  await expect(update).rejects.toThrow();
}

export async function observerBeforeAwaitUsingForOfDisposal() {
  const update = startOperation();
  for (await using _lock of locks) {
    void update.catch(() => void 0);
  }
  await expect(update).rejects.toThrow();
}

export async function matcherBeforeAwaitUsingForOfDisposal() {
  const update = startOperation();
  for (await using _lock of locks) {
    await expect(update).rejects.toThrow();
  }
}

export async function nestedContinueDoesNotExitAwaitUsingIteration(keepSpinning: boolean) {
  const update = startOperation();
  for (await using _lock of locks) {
    while (keepSpinning) continue;
    await expect(update).rejects.toThrow();
  }
}

export async function breakWithoutEnclosingBackedgeCannotReachMatcher() {
  const update = startOperation();
  for (await using _lock of locks) {
    break;
    await expect(update).rejects.toThrow();
  }
}

export async function breakExitingEnclosingLoopCannotReachMatcher(flag: boolean) {
  const update = startOperation();
  outer: while (flag) {
    for (await using _lock of locks) {
      break outer;
      await expect(update).rejects.toThrow();
    }
  }
}

export async function caughtThrowReturningCannotReachLaterMatcher(flag: boolean) {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    try {
      for (await using _lock of locks) {
        if (flag) throw new Error("expected");
        await expect(update).rejects.toThrow();
      }
    } catch {
      return;
    }
  }
}

export async function caughtThrowInsideResourceScopeDoesNotDispose(flag: boolean) {
  const update = startOperation();
  for (await using _lock of locks) {
    try {
      if (flag) throw new Error("expected");
    } catch {
      Math.random();
    }
    await expect(update).rejects.toThrow();
  }
}

export async function caughtThrowBeforeTerminalOuterStatementCannotBackedge() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    try {
      for (await using _lock of locks) {
        if (index === 0) throw new Error("expected");
        await expect(update).rejects.toThrow();
      }
    } catch {
      Math.random();
    }
    return;
  }
}

export async function caughtThrowBeforeMixedTerminalBranchesCannotBackedge(flag: boolean) {
  const update = startOperation();
  outer: for (let index = 0; index < 2; index += 1) {
    try {
      for (await using _lock of locks) {
        if (index === 0) throw new Error("expected");
        await expect(update).rejects.toThrow();
      }
    } catch {
      Math.random();
    }
    if (flag) return;
    else break outer;
  }
}

export async function caughtThrowWithoutEnclosingLoopCannotBackedge(flag: boolean) {
  const update = startOperation();
  try {
    for (await using _lock of locks) {
      if (flag) throw new Error("expected");
      await expect(update).rejects.toThrow();
    }
  } catch {
    Math.random();
  }
}

export async function caughtThrowBreakingEnclosingLoopCannotReachMatcher() {
  const update = startOperation();
  outer: for (;;) {
    try {
      for (await using _lock of locks) {
        throw new Error("expected");
        await expect(update).rejects.toThrow();
      }
    } catch {
      break outer;
    }
  }
}

export async function throwOverriddenByReturningFinallyCannotReachMatcher(flag: boolean) {
  const update = startOperation();
  for (await using _lock of locks) {
    try {
      if (flag) throw new Error("expected");
    } finally {
      return;
    }
    await expect(update).rejects.toThrow();
  }
}

export async function innerThrowingFinallyCannotReachMatcher(flag: boolean) {
  const update = startOperation();
  try {
    for (await using _lock of locks) {
      try {
        if (flag) throw new Error("initial");
      } finally {
        throw new Error("terminal");
      }
      await expect(update).rejects.toThrow();
    }
  } catch {
    Math.random();
  }
}

export async function uncaughtThrowCannotReachLaterMatcher(flag: boolean) {
  const update = startOperation();
  for (await using _lock of locks) {
    if (flag) throw new Error("expected");
    await expect(update).rejects.toThrow();
  }
}

export async function awaitUsingScopeExitAfterConditionalMatcher(flag: boolean) {
  const update = startOperation();
  if (flag) await expect(update).rejects.toThrow();
  {
    await using _lock = acquireLock();
  }
}

export async function observerInSwitchDiscriminantDominatesAwait() {
  const update = startOperation();
  switch (update.catch(() => void 0)) {
    default:
      Math.random();
  }
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInForInitializerDominatesAwait() {
  const update = startOperation();
  for (void update.catch(() => void 0); Math.random() > 0.5;) break;
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInWhileTestDominatesAwait() {
  const update = startOperation();
  while (update.catch(() => void 0)) break;
  await release();
  await expect(update).rejects.toThrow();
}

export async function forAwaitRightObserverDominatesLaterAwait() {
  const update = startOperation();
  for await (const _item of [update.catch(() => void 0)]) Math.random();
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInUnconditionalNestedBlock() {
  const update = startOperation();
  {
    void update.catch(() => void 0);
  }
  await release();
  await expect(update).rejects.toThrow();
}

export async function observerInFinallyDominatesLaterAwait() {
  const update = startOperation();
  try {
    Math.random();
  } finally {
    void update.catch(() => void 0);
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

export async function returningCatchDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    throw new Error("caught");
  } catch {
    return;
  }
  await expect(update).rejects.toThrow();
}

export async function bothTryBranchesReturnBeforeMatcher(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    if (flag) return;
    else return;
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function returningInnerFinallyDoesNotReachMatcher() {
  const update = startOperation();
  try {
    try {
      await release();
      throw new Error("replaced by return");
    } finally {
      return;
    }
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function enclosingReturningFinallyDoesNotReachMatcher(skip: boolean) {
  const update = startOperation();
  if (skip) {
    try {
      await release();
    } finally {
      return;
    }
  }
  await expect(update).rejects.toThrow();
}

export async function enclosingContinuingFinallyDoesNotReachMatcher(flag: boolean) {
  const update = startOperation();
  while (flag) {
    try {
      await release();
    } finally {
      continue;
    }
    await expect(update).rejects.toThrow();
  }
}

export async function enclosingLabeledContinueDoesNotReachMatcher(flag: boolean) {
  const update = startOperation();
  outer: while (flag) {
    while (flag) {
      try {
        await release();
      } finally {
        continue outer;
      }
      await expect(update).rejects.toThrow();
    }
  }
}

export async function abruptReturningInnerFinallyDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    try {
      throw new Error("replaced by return");
    } finally {
      return;
    }
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
}

export async function nestedReturningCatchDoesNotReachMatcher() {
  const update = startOperation();
  await release();
  try {
    try {
      throw new Error("caught");
    } catch {
      return;
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

export async function caughtReturnAwaitStillReturnsFromFunction() {
  const update = startOperation();
  try {
    return await release();
  } catch {
    return;
  }
  await expect(update).rejects.toThrow();
}

export async function returnAwaitOverriddenByFinallyDoesNotReachAssertion() {
  const update = startOperation();
  try {
    return await release();
  } catch {
    Math.random();
  } finally {
    return;
  }
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

export async function breakThroughFinallyDoesNotReachAssertion() {
  const update = startOperation();
  while (true) {
    await release();
    try {
      break;
    } finally {
      Math.random();
    }
    await expect(update).rejects.toThrow();
  }
}

export async function breakFromCatchDoesNotReachAssertion() {
  const update = startOperation();
  while (true) {
    await release();
    try {
      throw new Error("caught");
    } catch {
      break;
    } finally {
      Math.random();
    }
    await expect(update).rejects.toThrow();
  }
}

export async function suspensionCatchBreakDoesNotReachAssertion() {
  const update = startOperation();
  while (true) {
    try {
      await release();
      return;
    } catch {
      break;
    }
    await expect(update).rejects.toThrow();
  }
}

export async function suspensionFinallyBreakDoesNotReachAssertion() {
  const update = startOperation();
  while (true) {
    try {
      await release();
      return;
    } catch {
      Math.random();
    } finally {
      break;
    }
    await expect(update).rejects.toThrow();
  }
}

export async function breakFromFinallyDoesNotReachAssertion() {
  const update = startOperation();
  while (true) {
    await release();
    try {
      Math.random();
    } finally {
      break;
    }
    await expect(update).rejects.toThrow();
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

export async function allSettledObservesBeforeAwait() {
  const update = startOperation();
  await Promise.allSettled([update, release()]);
  await expect(update).rejects.toThrow();
}

export async function awaitedAllObservesBeforeAwait() {
  const update = startOperation();
  try {
    await Promise.all([update, release()]);
  } catch {
    Math.random();
  }
  await expect(update).rejects.toThrow();
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
