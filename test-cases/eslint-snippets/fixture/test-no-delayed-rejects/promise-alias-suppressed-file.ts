// no-mistakes-disable-file test-no-delayed-rejects -- reviewed timing
async function suppressedFile() {
  const p = work();
  const alias = p;
  await barrier();
  await expect(alias).rejects.toThrow();
}
