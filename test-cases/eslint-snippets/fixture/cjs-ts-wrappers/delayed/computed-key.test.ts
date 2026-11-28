const spec = "vitest";
const check = require(spec);
const rejects = "status";
async function reads() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await check(pending)[rejects];
}
