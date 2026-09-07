import { expect } from "@jest/globals";

declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function importedJestExpect() {
  const update = startOperation();
  await release();
  await expect(update).rejects.toThrow();
}
