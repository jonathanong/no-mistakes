import { expect } from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;
declare const dynamicProperty: string;

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
