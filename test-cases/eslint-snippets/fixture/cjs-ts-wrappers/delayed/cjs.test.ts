const { expect } = require("vitest");
const { expect: e } = require("vitest");
const vt = require("vitest");
import vt2 = require("vitest");
async function named() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await expect(pending).rejects.toThrow();
}
async function aliased() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await e(pending).rejects.toThrow();
}
async function namespace() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await vt.expect(pending).rejects.toThrow();
}
async function imported() {
  const pending = Promise.reject(new Error("x"));
  await Promise.resolve();
  await vt2.expect(pending).rejects.toThrow();
}
