declare function release(): Promise<void>;
declare function startOperation(): Promise<void>;

export async function globalExpect() {
  const update = startOperation();
  await release();
  await expect(update).rejects.toThrow();
}
