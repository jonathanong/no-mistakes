const spec = "vitest";
const vt = require(spec);
async function reads() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await vt.expect(pending).rejects.toThrow();
}
