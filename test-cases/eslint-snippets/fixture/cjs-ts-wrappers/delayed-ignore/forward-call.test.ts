const source = "vitest";
const check = require(source);
async function reads() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await check(pending).rejects.toThrow();
  await check.call(null, pending).rejects.toThrow();
  await check.apply(null, [pending]).rejects.toThrow();
}
