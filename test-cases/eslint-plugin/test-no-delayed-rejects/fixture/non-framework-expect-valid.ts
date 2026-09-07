import { expect } from "@custom/assertions";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function unsupportedImportedExpect() {
  const update = startOperation();
  await release();
  await expect(update).rejects.toThrow();
}
