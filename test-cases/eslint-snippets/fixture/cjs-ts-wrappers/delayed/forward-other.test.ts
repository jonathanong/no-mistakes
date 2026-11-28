const source = "vitest";
const check = require(source);
const other = (value) => value;
async function reads() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await check.call.call(other, null, pending).rejects.toThrow();
  await check.apply.call(other, null, [...pending]).rejects.toThrow();
}
