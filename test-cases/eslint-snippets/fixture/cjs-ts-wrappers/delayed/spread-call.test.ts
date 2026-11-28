const source = "vitest";
const { expect: check } = require(source);
async function reads() {
  const pending = Promise.reject(new Error("x"));
  const values = [pending];
  await Promise.resolve();
  await check.call(null, ...values).rejects.toThrow();
}
void reads;
