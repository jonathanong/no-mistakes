import { describe, expect, it, vi } from "vitest";
import { contextExtensionRunner } from "../../../fixtures/test-timeout-policy/context-added-hooks/runner.mjs";
import { protectRunnerTimeouts } from "./index.mjs";

describe("completion hooks added by custom context extensions", () => {
  it.each(["onTestFinished", "onTestFailed"])(
    "guards a newly added %s on the original context",
    (name) => {
      const original = vi.fn();
      const runner = contextExtensionRunner({ [name]: original });
      protectRunnerTimeouts(runner, 25);
      const context = { task: { timeout: 25 } };
      expect(runner.extendTaskContext(context)).toBe(context);
      expect(() => context[name](() => {}, 26)).toThrow(RangeError);
      expect(() => context[name](() => {}, 0)).toThrow(RangeError);
      expect(original).not.toHaveBeenCalled();
      const callback = () => {};
      context[name](callback, 25);
      expect(original).toHaveBeenCalledWith(callback, 25);
    },
  );

  it("rescans late second hooks while preserving installed methods and native restoration identity", () => {
    const hooks = { onTestFinished: vi.fn() };
    const runner = contextExtensionRunner(hooks);
    protectRunnerTimeouts(runner, 25);
    const context = { task: { timeout: 25 } };
    runner.extendTaskContext(context);
    const saved = context.onTestFinished;
    runner.extendTaskContext(context);
    expect(context.onTestFinished).toBe(saved);
    const temporary = vi.fn();
    context.onTestFinished = temporary;
    expect(() => context.onTestFinished(() => {}, 26)).toThrow(RangeError);
    expect(temporary).not.toHaveBeenCalled();
    context.onTestFinished = saved;
    expect(context.onTestFinished).toBe(saved);
    hooks.onTestFailed = vi.fn();
    runner.extendTaskContext(context);
    expect(context.onTestFinished).toBe(saved);
    expect(() => context.onTestFailed(() => {}, Infinity)).toThrow(RangeError);
    expect(hooks.onTestFailed).not.toHaveBeenCalled();
    expect(() => {
      context.onTestFailed = null;
    }).toThrow(TypeError);
    expect(Reflect.deleteProperty(context, "onTestFailed")).toBe(false);
    context.onTestFailed(() => {}, 25);
    expect(hooks.onTestFailed).toHaveBeenCalledOnce();
  });

  it("rejects a different maximum even when the first extension installs no methods", () => {
    const first = contextExtensionRunner({});
    const second = contextExtensionRunner({ onTestFailed: vi.fn() }, 20);
    protectRunnerTimeouts(first, 25);
    protectRunnerTimeouts(second, 20);
    const context = { task: { timeout: 20 } };
    first.extendTaskContext(context);
    expect(() => second.extendTaskContext(context)).toThrow(
      "Context timeout policy maximum cannot change",
    );
    expect(context.onTestFailed).toBeUndefined();
  });
});
