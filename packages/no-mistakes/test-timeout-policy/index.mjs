import { guardRegistration } from "./registration.mjs";
export { guardRegistration } from "./registration.mjs";
import { guardCollectionRunner } from "./public-runner.mjs";
export { guardPublicRunner } from "./public-runner.mjs";
import { assertTestTimeout, assertTimeoutConfig, protectTimeoutConfig } from "./config.mjs";
export {
  assertTestTimeout,
  assertTimeoutConfig,
  snapshotTimeoutOptions,
  protectTimeoutConfig,
} from "./config.mjs";
const installedRunners = new WeakMap();
const installedContexts = new WeakMap();
function protectContextHooks(context, maximum) {
  const installed = installedContexts.get(context) ?? { maximum, methods: new Set() };
  if (installed.maximum !== maximum)
    throw new Error("Context timeout policy maximum cannot change");
  for (const name of ["onTestFailed", "onTestFinished"]) {
    // Extensions may add a missing hook to the same context between scans.
    if (installed.methods.has(name)) continue;
    const original = Reflect.get(context, name);
    if (typeof original === "function") {
      let guarded = guardRegistration(original, "hook", maximum);
      Object.defineProperty(context, name, {
        // Vitest temporarily replaces/restores these methods while executing completion hooks.
        get: () => guarded,
        set(next) {
          if (typeof next !== "function") throw new TypeError("Expected a completion hook method");
          guarded = guardRegistration(next, "hook", maximum);
        },
        configurable: false,
      });
      installed.methods.add(name);
    }
  }
  installedContexts.set(context, installed);
}
/** Aliased/destructured setters retain validation because the returned method is guarded. */
export function guardRuntimeConfig(api, maximum = 30_000) {
  const original = Reflect.get(api, "setConfig");
  if (typeof original !== "function") throw new TypeError("Expected a Vitest setConfig method");
  const setter = (config) => {
    const snapshot = { ...config };
    assertTimeoutConfig(snapshot, maximum);
    return Reflect.apply(original, api, [snapshot]);
  };
  return new Proxy(api, {
    get(target, key, receiver) {
      return key === "setConfig" ? setter : Reflect.get(target, key, receiver);
    },
  });
}
/** A single current Vitest API facade. Adoption must route every test import through it. */
export function guardVitestExports(api, maximum = 30_000) {
  const cache = new Map();
  const kinds = new Map([
    ["test", "test"],
    ["it", "test"],
    ["describe", "suite"],
    ["suite", "suite"],
    ["beforeAll", "hook"],
    ["beforeEach", "hook"],
    ["afterAll", "hook"],
    ["afterEach", "hook"],
    ["aroundAll", "hook"],
    ["aroundEach", "hook"],
    ["onTestFailed", "hook"],
    ["onTestFinished", "hook"],
  ]);
  return new Proxy(api, {
    get(target, key, receiver) {
      const value = Reflect.get(target, key, receiver);
      const kind = kinds.get(String(key));
      const role = kind ?? (key === "vitest" ? "vi" : key);
      const values = cache.get(value) ?? new Map();
      if (values.has(role)) return values.get(role);
      const guarded =
        key === "TestRunner" && typeof value === "function"
          ? guardCollectionRunner(value, maximum, protectRunnerTimeouts)
          : (key === "vi" || key === "vitest") && value !== null && typeof value === "object"
            ? guardRuntimeConfig(value, maximum)
            : kind && typeof value === "function"
              ? guardRegistration(value, kind, maximum)
              : value;
      values.set(role, guarded);
      cache.set(value, values);
      return guarded;
    },
  });
}
/** Compose at runner construction, without replacing existing collection/database guards. */
export function protectRunnerTimeouts(runner, maximum = 30_000) {
  const existing = installedRunners.get(runner);
  if (existing) {
    if (existing.maximum !== maximum || existing.config !== runner.config)
      throw new Error("Installed runner timeout policy cannot be replaced");
    return;
  }
  if (Reflect.get(runner.config, "globals") === true)
    throw new Error("Timeout policy requires guarded public imports with globals disabled");
  protectTimeoutConfig(runner.config, maximum);
  const config = runner.config;
  const extend = Reflect.get(runner, "extendTaskContext");
  const extendTaskContext = (context) => {
    if (runner.config !== config)
      throw new Error("Runner timeout configuration identity changed after construction");
    // Vitest 5 creates the context after resolving task.timeout, before withTimeout.
    // withTimeout uses a pre-context local timeout; extensions must not mask invalid tasks.
    const task = Reflect.get(context, "task");
    if (task === null || typeof task !== "object")
      throw new TypeError("Expected Vitest task context");
    assertTestTimeout(Reflect.get(task, "timeout"), "Vitest resolved test timeout", maximum);
    protectContextHooks(context, maximum);
    const result =
      typeof extend === "function" ? Reflect.apply(extend, runner, [context]) : context;
    if (runner.config !== config)
      throw new Error("Runner timeout configuration identity changed during context extension");
    // Recheck the original task, even if the extension returns a replacement context.
    assertTestTimeout(Reflect.get(task, "timeout"), "Vitest resolved test timeout", maximum);
    protectContextHooks(result, maximum);
    return result;
  };
  Object.defineProperty(runner, "extendTaskContext", {
    value: extendTaskContext,
    writable: false,
    configurable: false,
  });
  installedRunners.set(runner, { maximum, config: runner.config });
}
