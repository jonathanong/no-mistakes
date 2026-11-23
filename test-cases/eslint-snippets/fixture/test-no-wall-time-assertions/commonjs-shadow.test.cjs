function check(require) {
  const { expect: verify, jest } = require("@jest/globals");
  jest.useFakeTimers();
  verify(Date.now() - start).toBeLessThan(200);
  expect(Date.now() - start).toBeLessThan(200);
}
