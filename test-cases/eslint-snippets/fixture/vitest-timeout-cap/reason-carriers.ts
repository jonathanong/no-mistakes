import { test, aroundAll, aroundEach, onTestFinished, onTestFailed, beforeAll, vi } from "vitest";
import { defineConfig } from "vitest/config";
aroundAll(async run => run(), 6000);
aroundEach(async run => run(), 6000);
onTestFinished(() => {}, 6000);
onTestFailed(() => {}, 6000);
beforeAll.call(undefined, () => {}, 6000);
beforeAll.apply(undefined, [() => {}, 6000]);
// no-mistakes-timeout-reason: isolated worker startup is slow
aroundAll(async run => run(), 6000);
// no-mistakes-timeout-reason: isolated worker startup is slow
aroundEach(async run => run(), 6000);
// no-mistakes-timeout-reason: waits for the remote worker cleanup
onTestFinished(() => {}, 6000);
// no-mistakes-timeout-reason: waits for the remote worker cleanup
onTestFailed(() => {}, 6000);
// no-mistakes-timeout-reason: isolated worker startup is slow
beforeAll.call(undefined, () => {}, 6000);
// no-mistakes-timeout-reason: isolated worker startup is slow
beforeAll.apply(undefined, [() => {}, 6000]);
test("context", context => {
  context.onTestFinished(() => {}, 6000);
  context.onTestFailed(() => {}, 6000);
  // no-mistakes-timeout-reason: waits for the worker cleanup
  context.onTestFinished(() => {}, 6000);
  // no-mistakes-timeout-reason: waits for the worker cleanup
  context.onTestFailed(() => {}, 6000);
});
// A reason on a config cannot lift the default cap.
export default defineConfig({ test: {
  // no-mistakes-timeout-reason: worker startup
  testTimeout: 6000,
  // no-mistakes-timeout-reason: worker startup
  hookTimeout: 6000,
} });
vi.setConfig({
  // no-mistakes-timeout-reason: worker startup
  testTimeout: 6000,
  hookTimeout: 6000,
});
// Superseded property reasons never justify replacement values.
const explained = { /* no-mistakes-timeout-reason: worker startup */ timeout: 6000 };
test("overwritten", { ...explained, timeout: 6000 }, () => {});
test("preserved", { ...explained }, () => {});
// no-mistakes-timeout-reason: worker startup
const scalar = 6000;
test("scalar", () => {}, scalar);
const options = { timeout: 6000 };
test("use annotation", /* no-mistakes-timeout-reason: worker startup */ options, () => {});
/*
 * no-mistakes-timeout-reason: worker startup
 */
test("multiline", () => {}, 6000);
// no-mistakes-timeout-reason: worker startup
// explanatory continuation
test("contiguous comments", () => {}, 6000);
// no-mistakes-timeout-reason: ...
test("punctuation", () => {}, 6000);
// no-mistakes-timeout-reason: <...>
test("angle punctuation", () => {}, 6000);
// no-mistakes-timeout-reason: reason
test("reason placeholder", () => {}, 6000);
// no-mistakes-timeout-reason: <reason>
test("angle reason", () => {}, 6000);
// no-mistakes-timeout-reason: N/A
test("not applicable", () => {}, 6000);
// no-mistakes-timeout-reason: None
test("none", () => {}, 6000);
