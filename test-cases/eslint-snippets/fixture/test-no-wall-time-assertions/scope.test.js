import { expect } from "vitest";

const outer = performance.now();
test("only follows local captures with unshadowed clocks", () => {
  const start = performance.now();
  {
    const start = 10;
    expect(start - offset).toBeLessThan(200);
  }
  expect(start - offset).toBeLessThan(200);
  expect(outer - offset).toBeLessThan(200);

  function localClock(Date, performance) {
    expect(Date.now() - offset).toBeLessThan(200);
    expect(performance.now() - offset).toBeLessThan(200);
  }
  function localExpect(expect) {
    expect(Date.now() - offset).toBeLessThan(200);
  }
  let mutable = performance.now();
  expect(mutable - offset).toBeLessThan(200);
  expect(later - offset).toBeLessThan(200);
  const later = performance.now();

  const elapsed = start - offset;
  function localFunction() {
    expect(start - offset).toBeLessThan(200);
    expect(elapsed).toBeLessThan(200);
  }
});
