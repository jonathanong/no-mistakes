import { expect as check } from "vitest";
import * as v from "vitest";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function bracket() {
  const update = startOperation();
  await release();
  await check(update)["rejects"].toThrow();
}

export async function template() {
  const update = startOperation();
  await release();
  await v.expect(update)[`rejects`].toThrow();
}

export async function optional() {
  const update = startOperation();
  await release();
  await (check?.(update).rejects).toThrow();
}
