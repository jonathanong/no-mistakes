import { expect } from "vitest";

declare function release(): Promise<void>;
declare function loadExpected(): Promise<unknown>;
declare function loadHandler(): Promise<(value: void) => void>;
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

export async function suspensionAfterPromiseCreationInsideInitializer() {
  const update = startOperation().then(await loadHandler());
  await expect(update).rejects.toThrow();
}

export async function branch() {
  const update = startOperation();
  if (Math.random() > 0.5) await release();
  await expect(update).rejects.toThrow();
}

export async function conditionalEarlierMatcherDoesNotDominate(flag: boolean) {
  const update = startOperation();
  if (flag) await expect(update).rejects.toThrow();
  await release();
  await expect(update).rejects.toThrow();
}

export async function switchBreakStillReachesLaterMatcher(flag: boolean) {
  const update = startOperation();
  switch (flag) {
    case true:
      await release();
      break;
  }
  await expect(update).rejects.toThrow();
}

export async function switchFallthroughReachesMatcher(kind: "wait" | "assert") {
  const update = startOperation();
  switch (kind) {
    case "wait":
      await release();
      Math.random();
    case "assert":
      await expect(update).rejects.toThrow();
      break;
  }
}

export async function breakAfterMatcherDoesNotSuppressDiagnostic() {
  const update = startOperation();
  while (true) {
    await release();
    await expect(update).rejects.toThrow();
    break;
  }
}

export async function nestedBreakAfterMatcherDoesNotSuppressDiagnostic() {
  const update = startOperation();
  while (true) {
    await release();
    await expect(update).rejects.toThrow();
    {
      break;
    }
  }
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

export async function forAwaitSuspendsBeforeBodyMatcher(items: AsyncIterable<unknown>) {
  const update = startOperation();
  for await (const _item of items) {
    await expect(update).rejects.toThrow();
  }
}

export async function forAwaitHandlerDoesNotObserveBeforeIteration(items: AsyncIterable<unknown>) {
  const update = startOperation();
  for await (const _item of items) {
    void update.catch(() => void 0);
    await expect(update).rejects.toThrow();
  }
}

export async function forAwaitBindingHandlerDoesNotObserveBeforeIteration(
  items: AsyncIterable<unknown>,
) {
  const update = startOperation();
  for await (const [_item = update.catch(() => void 0)] of items) {
    await expect(update).rejects.toThrow();
  }
}

export async function loopBackedgeReachesOtherBranch() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    if (index === 0) await release();
    else await expect(update).rejects.toThrow();
  }
}

export async function reversedLoopBackedgeReachesEarlierBranch() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    if (index === 1) await expect(update).rejects.toThrow();
    else await release();
  }
}

export async function switchBackedgeReachesEarlierCase() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    switch (index) {
      case 1:
        await expect(update).rejects.toThrow();
        break;
      default:
        await release();
    }
  }
}

export async function skippedMatcherLetsLaterAwaitSuspendFirst() {
  const update = startOperation();
  for (let index = 0; index < 2; index += 1) {
    if (index === 1) await expect(update).rejects.toThrow();
    await release();
  }
}

export async function forAwaitSuspendsBeforeBindingMatcher(items: AsyncIterable<unknown>) {
  const update = startOperation();
  for await (const [_item = await expect(update).rejects.toThrow()] of items) {
    // The iterator suspends before evaluating the binding default.
  }
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

export async function conditionalCatchInsideAwaitDoesNotDominate(flag: boolean) {
  const update = startOperation();
  await (flag && update.catch(() => undefined));
  await expect(update).rejects.toThrow();
}

export async function absentCatchHandler() {
  const update = startOperation();
  void update.catch(undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function shadowedUndefinedHandlerCanRejectChild() {
  const undefined = Promise.reject(new Error("child"));
  const update = startOperation();
  void update.catch(() => undefined);
  await release();
  await expect(update).rejects.toThrow();
}

export async function rejectionReasonThenableCanRejectChild() {
  const update = startOperation();
  void update.catch((reason: unknown) => reason);
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

export async function unsafeContinuationAfterSafeCatch() {
  const update = startOperation();
  void update
    .catch((error: unknown) => error)
    .then((error) => {
      throw error;
    });
  await release();
  await expect(update).rejects.toThrow();
}

export async function wrappedUnsafeContinuationAfterSafeCatch() {
  const update = startOperation();
  void (update.catch((error: unknown) => error) as Promise<unknown>).then(() => {
    throw new Error("expected");
  });
  await release();
  await expect(update).rejects.toThrow();
}

export async function destructuredCatchHandlerIsNotProvablySafe() {
  const update = startOperation();
  void update.catch(({ message }: Error) => message);
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

export async function compoundThrowReachesCatchMatcher(flag: boolean) {
  const update = startOperation();
  try {
    await release();
    {
      if (flag) throw new Error("first");
      else throw new Error("second");
    }
  } catch {
    await expect(update).rejects.toThrow();
  }
}

export async function caughtThrowContinuesToLaterMatcher() {
  const update = startOperation();
  try {
    await release();
    throw new Error("caught");
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function compoundCaughtThrowContinuesToLaterMatcher(flag: boolean) {
  const update = startOperation();
  try {
    await release();
    if (flag) throw new Error("first");
    else throw new Error("second");
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function innerRethrowReachesOuterCatchAndMatcher() {
  const update = startOperation();
  try {
    try {
      await release();
      throw new Error("inner");
    } catch {
      throw new Error("outer");
    }
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function throwingInnerFinallyReachesOuterCatchAndMatcher() {
  const update = startOperation();
  try {
    await release();
    try {
      Math.random();
    } finally {
      throw new Error("outer");
    }
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function suspensionInsideThrowingFinallyFlowReachesMatcher() {
  const update = startOperation();
  try {
    try {
      await release();
      throw new Error("replaced");
    } finally {
      throw new Error("caught outside");
    }
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function nestedCaughtThrowContinuesToLaterMatcher() {
  const update = startOperation();
  try {
    try {
      await release();
      throw new Error("caught outside");
    } finally {
      // The throw continues through this non-abrupt finalizer.
    }
  } catch {
    // Execution continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function returnAwaitRejectionReachesCatchAndMatcher() {
  const update = startOperation();
  try {
    return await release();
  } catch {
    // A rejection continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function ordinaryAwaitRejectionReachesCatchPastReturn() {
  const update = startOperation();
  try {
    await release();
    return;
  } catch {
    // A rejection continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function returnAwaitRejectionReachesMatcherInCatch() {
  const update = startOperation();
  try {
    return await release();
  } catch {
    await expect(update).rejects.toThrow();
  }
}

export async function returnAwaitRethrowReachesOuterCatchAndMatcher() {
  const update = startOperation();
  try {
    try {
      return await release();
    } catch {
      throw new Error("outer");
    }
  } catch {
    // The rethrow continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function conditionalCaughtThrowBypassesLaterReturn(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    if (flag) throw new Error("caught");
    return;
  } catch {
    // The conditional throw continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function caughtConditionalThrowOrReturnReachesMatcher(flag: boolean) {
  const update = startOperation();
  try {
    await release();
    if (flag) throw new Error("caught");
    else return;
  } catch {
    // The throwing branch continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function conditionalThrowReachesMatcherInCatch(flag: boolean) {
  const update = startOperation();
  try {
    await release();
    if (flag) throw new Error("caught");
    return;
  } catch {
    await expect(update).rejects.toThrow();
  }
}

export async function switchThrowBypassesLaterReturn(kind: "throw" | "return") {
  const update = startOperation();
  await release();
  try {
    switch (kind) {
      case "throw":
        throw new Error("caught");
      default:
        break;
    }
    return;
  } catch {
    // The throwing switch case continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function nestedConditionalRethrowBypassesLaterReturn(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    try {
      if (flag) throw new Error("inner");
    } catch {
      throw new Error("outer");
    }
    return;
  } catch {
    // The nested throw continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function conditionalThrowThroughThrowingFinally(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    try {
      if (flag) throw new Error("inner");
      return;
    } finally {
      throw new Error("outer");
    }
  } catch {
    // The finalizer throw continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function conditionalFinalizerThrowBypassesLaterReturn(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    try {
      return;
    } finally {
      if (flag) throw new Error("caught");
    }
    return;
  } catch {
    // The finalizer's throwing branch continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function conditionalThrowThroughRethrowingCatch(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    try {
      if (flag) throw new Error("inner");
      return;
    } catch {
      throw new Error("outer");
    }
  } catch {
    // The rethrow continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function loopThrowBypassesLaterReturn(flag: boolean) {
  const update = startOperation();
  await release();
  try {
    while (flag) throw new Error("caught");
    return;
  } catch {
    // The throwing loop body continues to the matcher below.
  }
  await expect(update).rejects.toThrow();
}

export async function* yieldSuspendsBeforeMatcher() {
  const update = startOperation();
  yield "ready";
  await expect(update).rejects.toThrow();
}
