import { withGeneratedFacade } from "../../../fixtures/test-timeout-policy/generated-facade.mjs";
import { describe, expect, it, vi, TestRunner } from "vitest";
import {
  assertTestTimeout,
  assertTimeoutConfig,
  snapshotTimeoutOptions,
  protectTimeoutConfig,
  guardRegistration,
  guardRuntimeConfig,
  guardVitestExports,
  protectRunnerTimeouts,
} from "./index.mjs";
import { browserTestTimeoutPolicyPlugin, testTimeoutPolicyPlugin } from "./plugin.mjs";
describe("public deadline policy contract boundaries", () => {
  it.each([0, -1, NaN, Infinity, "30000", 30001])(
    "rejects invalid configured value %s",
    (value) => {
      expect(() => assertTestTimeout(value, "test timeout")).toThrow(RangeError);
    },
  );
  it("accepts the inclusive deadline and rejects invalid policy maxima", () => {
    expect(assertTestTimeout(30000, "test timeout")).toBe(30000);
    for (const maximum of [0, NaN, Infinity])
      expect(() => assertTestTimeout(1, "test timeout", maximum)).toThrow(RangeError);
    expect(() => assertTimeoutConfig({})).not.toThrow();
    expect(() => assertTimeoutConfig({ hookTimeout: 0 })).toThrow(RangeError);
  });
  it("captures timeout getters once while retaining other getter receivers", () => {
    class Options {
      #label = "owned";
      reads = 0;
      get timeout() {
        return ++this.reads === 1 ? 25 : 30001;
      }
      get label() {
        return this.#label;
      }
    }
    const options = new Options();
    Object.defineProperty(options, "timeout", {
      enumerable: true,
      get: () => Reflect.get(Options.prototype, "timeout", options),
    });
    const captured = snapshotTimeoutOptions(options, "test timeout");
    expect(Reflect.get(captured, "timeout")).toBe(25);
    expect(Reflect.get(captured, "timeout")).toBe(25);
    expect(Reflect.get(captured, "label")).toBe("owned");
    expect(options.reads).toBe(1);
    expect(() => snapshotTimeoutOptions({ timeout: null }, "test")).toThrow(RangeError);
    expect(
      Reflect.get(snapshotTimeoutOptions({ timeout: null }, "test", 30000, true), "timeout"),
    ).toBeNull();
    expect(Reflect.get(snapshotTimeoutOptions({}, "test"), "timeout")).toBeUndefined();
  });
  it("protects one config object across repeated installation and invalid writes", () => {
    const config = { testTimeout: 25, hookTimeout: 30 };
    protectTimeoutConfig(config);
    protectTimeoutConfig(config);
    expect(() => protectTimeoutConfig(config, 100)).toThrow("maximum cannot change");
    config.testTimeout = 30000;
    config.hookTimeout = 50;
    expect(() => {
      config.testTimeout = 0;
    }).toThrow(RangeError);
    expect(config.testTimeout).toBe(30000);
    expect(config.hookTimeout).toBe(50);
    expect(() => Object.defineProperty(config, "hookTimeout", { value: 0 })).toThrow(TypeError);
  });
  it("retains registration receiver, return value, optional args and unknown callable identity", () => {
    const receiver = { owned: true };
    const original = vi.fn(function (...args) {
      return { receiver: this, args };
    });
    const fn = () => "SDK callable metadata";
    const register = Object.assign(original, { fn, label: "preserved" });
    Object.defineProperty(register, "fn", { value: fn, writable: false, configurable: false });
    const guarded = guardRegistration(register, "test");
    const options = { timeout: 25 };
    const result = guarded.call(receiver, "name", options, () => {});
    expect(result.receiver).toBe(receiver);
    expect(result.args[0]).toBe("name");
    expect(Reflect.get(result.args[1], "timeout")).toBe(25);
    expect(guarded.fn).toBe(fn);
    expect(guarded.label).toBe("preserved");
    expect(() => guarded("name", 30001)).toThrow(RangeError);
    expect(() => guarded("name", null)).not.toThrow();
    const hook = guardRegistration(original, "hook");
    expect(() => hook(() => {}, 0)).toThrow(RangeError);
    expect(() => hook(() => {})).not.toThrow();
  });
  it("guards chained, curried and hook properties while retaining cache identity", () => {
    const target = (...args) => args;
    const methods = Object.assign(target, {
      shuffle: target,
      describe: target,
      beforeAll: target,
      each: () => target,
      runIf: () => target,
      extend: () => ({ opaque: true }),
    });
    const guarded = guardRegistration(methods, "suite");
    expect(guarded.shuffle).toBe(guarded.shuffle);
    expect(() => guarded.shuffle("name", 30001)).toThrow(RangeError);
    expect(() => guarded.describe("name", { timeout: 30001 })).toThrow(RangeError);
    expect(() => guarded.beforeAll(() => {}, 0)).toThrow(RangeError);
    expect(() => guarded.each()("name", 30001)).toThrow(RangeError);
    expect(() => guarded.runIf()("name", 30001)).toThrow(RangeError);
    expect(guarded.extend()).toEqual({ opaque: true });
  });
  it("preserves asynchronous suite-factory receivers and return values while protecting injected APIs", async () => {
    const receiver = { owner: "native factory" };
    const injected = Object.assign(vi.fn(), { beforeAll: vi.fn() });
    const original = (...args) => {
      const factory = args.find((value) => typeof value === "function");
      if (typeof factory !== "function") throw new Error("factory absent");
      return Reflect.apply(factory, receiver, [injected]);
    };
    const suite = guardRegistration(original, "suite");
    const result = suite("owned", { timeout: 25 }, async function (test) {
      expect(this).toBe(receiver);
      await Promise.resolve();
      expect(() => test.beforeAll(() => {}, 30001)).toThrow(RangeError);
      test.beforeAll(() => {}, 25);
      test("body", () => {}, 25);
      return "factory completed";
    });
    await expect(result).resolves.toBe("factory completed");
    expect(injected.beforeAll).toHaveBeenCalledOnce();
    expect(injected).toHaveBeenCalledOnce();
    const row = Object.assign(() => "data", { beforeAll() {} });
    const registration = Object.assign(() => {}, {
      each: () => (_name, factory) => factory(row),
    });
    const table = guardRegistration(registration, "suite").each();
    expect(
      table("row", (value) => {
        expect(value).toBe(row);
        return value();
      }),
    ).toBe("data");
  });
  it("rejects returned collector hooks while preserving native collection methods and cache identity", () => {
    const collector = {
      type: "collector",
      name: "owned",
      on: vi.fn(),
      test: vi.fn(),
      task: vi.fn(),
      collect: vi.fn(),
      clear: vi.fn(),
      tasks: [],
    };
    const register = guardRegistration(() => collector, "suite");
    const guarded = register();
    expect(register()).toBe(guarded);
    expect(guarded.on).toBe(guarded.on);
    expect(() => guarded.on("beforeAll", () => {})).toThrow("register guarded public hooks");
    expect(collector.on).not.toHaveBeenCalled();
    expect(guarded.test).toBe(guarded.test);
    expect(guarded.test).not.toBe(collector.test);
    expect(() => guarded.test("invalid", 30001)).toThrow(RangeError);
    for (const key of ["name", "task", "collect", "clear", "tasks"])
      expect(Reflect.get(guarded, key)).toBe(Reflect.get(collector, key));
    guarded.clear();
    expect(collector.clear).toHaveBeenCalledOnce();
  });
  it("snapshots aliased setter options and preserves configuration API receiver", () => {
    const api = {
      value: 25,
      setConfig(config) {
        this.value = Reflect.get(config, "testTimeout");
        return "applied";
      },
    };
    const wrapped = guardRuntimeConfig(api);
    // Runtime alias extraction deliberately carries no receiver; the guard must preserve it.
    const setConfig = Reflect.get(wrapped, "setConfig");
    if (typeof setConfig !== "function") throw new Error("Configuration setter missing");
    let reads = 0;
    expect(
      setConfig({
        get testTimeout() {
          return ++reads === 1 ? 30 : 30001;
        },
      }),
    ).toBe("applied");
    expect(api.value).toBe(30);
    expect(reads).toBe(1);
    expect(() => setConfig({ testTimeout: 0 })).toThrow(RangeError);
    expect(wrapped.value).toBe(30);
    expect(() => guardRuntimeConfig({})).toThrow(TypeError);
  });
  it("caches guarded public exports and preserves unrelated values", () => {
    const register = vi.fn();
    const api = { test: register, beforeAll: register, vi: { setConfig: vi.fn() }, value: null };
    const guarded = guardVitestExports(api);
    expect(guarded.test).toBe(guarded.test);
    expect(guarded.vi).toBe(guarded.vi);
    expect(guarded.value).toBeNull();
    expect(() => guarded.test("name", 30001)).toThrow(RangeError);
    expect(() => guarded.beforeAll(() => {}, 0)).toThrow(RangeError);
    expect(() => guarded.vi.setConfig({ hookTimeout: Infinity })).toThrow(RangeError);
  });
  it("guards constructor exports in the parent while preserving constructor and instance identity", () => {
    class Runner {
      config;
      #marker = "owned";
      constructor(config) {
        this.config = config;
      }
      marker() {
        return this.#marker;
      }
    }
    const facade = guardVitestExports({ TestRunner: Runner });
    expect(facade.TestRunner).toBe(facade.TestRunner);
    const runner = new facade.TestRunner({ testTimeout: 25, hookTimeout: 25 });
    expect(runner).toBeInstanceOf(Runner);
    expect(runner.marker()).toBe("owned");
    expect(() => {
      runner.config.testTimeout = 30001;
    }).toThrow(RangeError);
    expect(() => new facade.TestRunner({ testTimeout: 0, hookTimeout: 25 })).toThrow(RangeError);
    expect(guardVitestExports({ TestRunner: null }).TestRunner).toBeNull();
  });
  it.each([false, true])(
    "worker-loads the public facade with the owning constructor policy; browser=%s",
    async (browser) => {
      const plugin = browser
        ? browserTestTimeoutPolicyPlugin()
        : testTimeoutPolicyPlugin(30000, [], { runner: null });
      const id = plugin.resolveId("vitest", "/owned/case.mts");
      await withGeneratedFacade(plugin.load(id), async (facade) => {
        expect(facade.TestRunner).not.toBe(TestRunner);
        expect(facade.TestRunner.prototype).toBe(TestRunner.prototype);
        class Custom extends facade.TestRunner {}
        expect(Object.getPrototypeOf(Custom.prototype)).toBe(TestRunner.prototype);
      });
    },
  );
  it("composes runner context and completion callbacks without masking raw invalid tasks", () => {
    const extension = vi.fn((context) => context);
    const runner = { config: { testTimeout: 25, hookTimeout: 25 }, extendTaskContext: extension };
    protectRunnerTimeouts(runner);
    protectRunnerTimeouts(runner);
    expect(() => protectRunnerTimeouts(runner, 100)).toThrow("cannot be replaced");
    expect(() => runner.extendTaskContext({ task: { timeout: 0 } })).toThrow(RangeError);
    expect(extension).not.toHaveBeenCalled();
    const callback = vi.fn();
    const context = { task: { timeout: 25 }, onTestFinished: callback };
    expect(runner.extendTaskContext(context)).toBe(context);
    expect(extension).toHaveBeenCalledWith(context);
    expect(() => context.onTestFinished(() => {}, 0)).toThrow(RangeError);
    context.onTestFinished(() => {}, 25);
    expect(callback).toHaveBeenCalledOnce();
    const replacement = vi.fn();
    expect(() => Reflect.set(context, "onTestFinished", null)).toThrow(TypeError);
    Reflect.set(context, "onTestFinished", replacement);
    expect(() => context.onTestFinished(() => {}, 30001)).toThrow(RangeError);
    expect(replacement).not.toHaveBeenCalled();
    context.onTestFinished(() => {}, 25);
    expect(replacement).toHaveBeenCalledOnce();
    expect(() => runner.extendTaskContext({ task: null })).toThrow(TypeError);
  });
  it("rejects post-extension mutation of the original task and supports context-only adapters", () => {
    const runner = {
      config: { testTimeout: 25, hookTimeout: 25 },
      extendTaskContext(context) {
        context.task.timeout = 30001;
        return { task: { timeout: 25 } };
      },
    };
    protectRunnerTimeouts(runner);
    expect(() => runner.extendTaskContext({ task: { timeout: 25 } })).toThrow(RangeError);
    const simple = { config: { testTimeout: 25, hookTimeout: 25 } };
    protectRunnerTimeouts(simple);
    const context = { task: { timeout: 25 } };
    expect(Reflect.apply(Reflect.get(simple, "extendTaskContext"), simple, [context])).toBe(
      context,
    );
    expect(() => protectRunnerTimeouts({ config: { globals: true } })).toThrow("globals disabled");
    simple.config = { testTimeout: 25, hookTimeout: 25 };
    expect(() => protectRunnerTimeouts(simple)).toThrow("cannot be replaced");
  });
  it("rejects reusing a context across runners with incompatible deadline policies", () => {
    const context = { task: { timeout: 10 }, onTestFinished: vi.fn() };
    const runner = { config: { testTimeout: 25, hookTimeout: 25 } };
    protectRunnerTimeouts(runner, 25);
    expect(Reflect.apply(Reflect.get(runner, "extendTaskContext"), runner, [context])).toBe(
      context,
    );
    const other = { config: { testTimeout: 20, hookTimeout: 20 } };
    protectRunnerTimeouts(other, 20);
    expect(() => Reflect.apply(Reflect.get(other, "extendTaskContext"), other, [context])).toThrow(
      "Context timeout policy maximum cannot change",
    );
  });
  it("routes only public Vitest imports and explicitly declares registrar inlining", () => {
    const plugin = testTimeoutPolicyPlugin(30000, ["owned-registrar"]);
    expect(plugin.config().test.server.deps.inline).toEqual(["owned-registrar"]);
    expect(() => testTimeoutPolicyPlugin(0)).toThrow(RangeError);
    expect(plugin.resolveId("other", "/case.mts")).toBeNull();
    expect(plugin.resolveId("vitest")).toBeNull();
    const id = plugin.resolveId("vitest", "/node_modules/owned-registrar/index.mjs");
    expect(plugin.resolveId("vitest", id)).toBeNull();
    expect(plugin.load("other")).toBeNull();
    // Vite load must supply module code for the owned route; SDK children prove its behavior.
    expect(plugin.load(id)).toBeTypeOf("string");
    // Default Node usage installs the native final runner and rejects unsupplied custom runners.
    const runner = plugin.config().test.runner;
    expect(() => plugin.configResolved({ test: { runner } })).not.toThrow();
    expect(() => plugin.config({ test: { runner: "/owned/existing-runner.mjs" } })).toThrow(
      "differs",
    );
  });
  it("keeps final Node runner provenance and rejects conflicting merged or browser slots", () => {
    const plugin = testTimeoutPolicyPlugin(30000, [], { runner: "/owned/final-runner.mjs" });
    const configured = plugin.config({ test: { runner: "/owned/final-runner.mjs" } });
    const runner = configured.test.runner;
    expect(plugin.resolveId(runner)).toBe(runner);
    expect(plugin.load(runner)).toBeTypeOf("string");
    expect(plugin.resolveId("vitest", runner)).toBeNull();
    expect(() => plugin.config({ test: { runner: "/other/runner.mjs" } })).toThrow("differs");
    expect(() => plugin.configResolved({ test: { runner } })).not.toThrow();
    expect(() => plugin.configResolved({ test: { runner: "/overridden.mjs" } })).toThrow(
      "replaced",
    );
    expect(() => plugin.configResolved({ test: { runner, browser: { enabled: true } } })).toThrow(
      "browser",
    );
    expect(() => testTimeoutPolicyPlugin(30000, [], { runner: "./relative.mjs" })).toThrow(
      "absolute",
    );
    const native = testTimeoutPolicyPlugin(30000, [], { runner: null });
    expect(native.load(native.config().test.runner)).toBeTypeOf("string");
  });
});
