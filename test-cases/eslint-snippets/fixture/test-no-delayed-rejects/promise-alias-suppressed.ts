async function suppressedNextLine() {
  const p = work();
  const alias = p;
  await barrier();
  // no-mistakes-disable-next-line test-no-delayed-rejects -- reviewed timing
  await expect(alias).rejects.toThrow();
}
async function suppressedLine() {
  const p = work();
  const alias = p;
  await barrier();
  await expect(alias).rejects.toThrow(); // no-mistakes-disable-line test-no-delayed-rejects -- reviewed timing
}
