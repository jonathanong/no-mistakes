import { describe, expect, it, vi } from "vitest";
import { BareRunner } from "../../../fixtures/test-timeout-policy/derived-fields/bare-runner.mjs";
import {
  BaseRunner,
  withBothFields,
  withExtensionField,
} from "../../../fixtures/test-timeout-policy/derived-fields/runners.mjs";
import { guardCollectionRunner, guardPublicRunner } from "./public-runner.mjs";
import { guardVitestExports, protectRunnerTimeouts } from "./index.mjs";

const config = () => ({ testTimeout: 25, hookTimeout: 25 });

describe("derived runner field initialization", () => {
  it("preserves both lifecycle fields and brands through a guarded base and final constructor", () => {
    const Base = guardPublicRunner(BaseRunner, 30_000, protectRunnerTimeouts);
    const Derived = withBothFields(Base);
    const Final = guardPublicRunner(Derived, 30_000, protectRunnerTimeouts);
    const runner = new Final(config());
    expect(runner.target).toBe(Derived);
    expect(runner).toBeInstanceOf(Derived);
    expect(runner).toBeInstanceOf(BaseRunner);
    expect(runner.onBeforeCollect(["test.mjs"])).toEqual(["base", "collection"]);
    expect(runner.paths).toEqual(["test.mjs"]);
    const context = { task: { timeout: 25 } };
    expect(runner.extendTaskContext(context)).toBe(context);
    expect(context.identities).toEqual(["base", "derived"]);
    for (const timeout of [0, -1, Infinity, NaN, 30_001]) {
      expect(() => runner.extendTaskContext({ task: { timeout } })).toThrow(RangeError);
      for (const key of ["testTimeout", "hookTimeout"]) {
        expect(() => {
          runner.config[key] = timeout;
        }).toThrow(RangeError);
      }
    }
    expect(() => {
      runner.extendTaskContext = (context) => context;
    }).toThrow(TypeError);
    const hooks = { task: { timeout: 25 }, onTestFinished() {}, onTestFailed() {} };
    runner.extendTaskContext(hooks);
    expect(() => hooks.onTestFinished(() => {}, 30_001)).toThrow(RangeError);
    expect(() => hooks.onTestFailed(() => {}, 0)).toThrow(RangeError);
  });

  it("installs SDK collection protection after extension fields initialize", () => {
    const Base = guardVitestExports({ TestRunner: BaseRunner }, 30_000).TestRunner;
    const Derived = withExtensionField(Base);
    const runner = new Derived(config());
    expect(runner.target).toBe(Derived);
    expect(runner.onBeforeCollect(["browser.mjs"])).toBe("base");
    expect(runner.paths).toEqual(["browser.mjs"]);
    const context = { task: { timeout: 25 } };
    expect(runner.extendTaskContext(context)).toBe(context);
    expect(context.identities).toEqual(["base", "derived"]);
    expect(() => runner.extendTaskContext({ task: { timeout: 30_001 } })).toThrow(RangeError);
    expect(() => {
      runner.config.hookTimeout = 0;
    }).toThrow(RangeError);
    expect(() => {
      runner.onBeforeCollect = () => {};
    }).toThrow(TypeError);
    expect(Reflect.deleteProperty(runner, "onBeforeCollect")).toBe(false);
  });

  it("protects a collection subclass that has no onBeforeCollect", () => {
    const protect = vi.fn();
    const Guarded = guardCollectionRunner(BareRunner, 25, protect);
    class Derived extends Guarded {}
    const runner = new Derived(config());
    expect(protect).not.toHaveBeenCalled();
    expect(runner.onBeforeCollect(["bare.mjs"])).toBeUndefined();
    expect(protect).toHaveBeenCalledOnce();
    expect(protect.mock.calls[0][0]).toBe(runner);
  });

  it("fails closed when a derived SDK runner replaces the collection hook with an own field", () => {
    const Base = guardVitestExports({ TestRunner: BaseRunner }, 30_000).TestRunner;
    const Derived = withBothFields(Base);
    // Sealing must reject initialization instead of silently skipping enforcement.
    expect(() => new Derived(config())).toThrow(TypeError);
  });
});
