test("asserts on elapsed time", async () => {
  const start = Date.now();
  await run();
  expect(Date.now() - start).toBeLessThan(200);
  expect(0 - performance.now()).toBeGreaterThan(-200);
  expect(Date.now()).toBeLessThan(200);
  expect(Date.now() - start).toBe(200);
  expect(Date.now() + start).toBeLessThan(200);
  expect(current - previous).toBeLessThan(200);
});
