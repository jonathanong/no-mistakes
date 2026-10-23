import { expect as check } from "vitest";

async function directAlias() {
  const p = work();
  const alias = p;
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function chainedAlias() {
  const p = work();
  const one = p;
  const two = one;
  await barrier();
  await expect(two).rejects.toThrow();
}

async function aliasAfterBarrier() {
  const p = work();
  await barrier();
  const alias = p;
  await expect(alias).rejects.toThrow();
}

const modulePromise = work();
await barrier();
const moduleAlias = modulePromise;
await expect(moduleAlias).rejects.toThrow();

async function forwardedExpect() {
  const p = work();
  const alias = p;
  await barrier();
  await check.call(null, alias)["rejects"].toThrow();
}

async function appliedExpect() {
  const p = work();
  const alias = p;
  await barrier();
  await check.apply(null, [alias]).rejects.toThrow();
}

async function staticMatcherAndAssertion() {
  const p = work();
  const alias = (p as Promise<unknown>)!;
  await barrier();
  await expect(alias)["rejects"].toThrow();
}

async function unsafeEarlyHandler() {
  const p = work();
  const alias = p;
  alias.catch((error) => {
    throw error;
  });
  await barrier();
  await expect(p).rejects.toThrow();
}

async function conditionalEarlyHandler(flag: boolean) {
  const p = work();
  const alias = p;
  if (flag) alias.catch(() => {});
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function sameStatementAlias() {
  const p = work(),
    alias = p;
  await barrier();
  await expect(alias).rejects.toThrow();
}
