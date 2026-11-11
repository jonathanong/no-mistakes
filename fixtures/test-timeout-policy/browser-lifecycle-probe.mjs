import { appendFileSync } from "node:fs";
import { performance } from "node:perf_hooks";

const wrappedReporters = new WeakSet();
const wrappedProviders = new WeakSet();

function errorDetails(error) {
  return { errorName: error instanceof Error ? error.name : typeof error };
}
function observeCompletion(result, phase, details) {
  let then;
  try {
    then = result?.then;
  } catch {
    recordBrowserLifecycle(`${phase}:completion-observer-error`, details);
    return true;
  }
  if (typeof then !== "function") return false;
  try {
    then.call(
      result,
      () => recordBrowserLifecycle(`${phase}:complete`, details),
      (error) => recordBrowserLifecycle(`${phase}:error`, { ...details, ...errorDetails(error) }),
    );
  } catch {
    recordBrowserLifecycle(`${phase}:completion-observer-error`, details);
  }
  return true;
}

/** Write a bounded, best-effort lifecycle record without affecting test behavior. */
export function recordBrowserLifecycle(phase, details = {}) {
  try {
    const record = {
      phase,
      at: new Date().toISOString(),
      atMs: performance.now(),
      pid: process.pid,
      ...details,
    };
    appendFileSync(
      process.env.BROWSER_LIFECYCLE_RECEIPT_FILE || "lifecycle.jsonl",
      `${JSON.stringify(record)}\n`,
    );
  } catch {
    // Diagnostic output must not change the reporter or policy result.
  }
}

function hookDetails(name, args) {
  const [value] = args;
  if (name === "onBrowserInit") return { projectName: value?.name ?? null };
  if (name === "onTestRunStart")
    return { specificationCount: Array.isArray(value) ? value.length : null };
  if (
    name === "onTestModuleCollected" ||
    name === "onTestModuleStart" ||
    name === "onTestModuleEnd"
  ) {
    return { moduleId: value?.moduleId ?? value?.file?.filepath ?? null };
  }
  if (name === "onTestCaseReady" || name === "onTestCaseResult") {
    let testState = null;
    try {
      testState =
        typeof value?.result === "function"
          ? (value.result()?.state ?? null)
          : (value?.result?.state ?? null);
    } catch {
      // Test state is supplemental; don't alter a reporter callback's behavior.
    }
    return { testName: value?.name ?? null, testState };
  }
  if (name === "onTestRunEnd")
    return { moduleCount: Array.isArray(value) ? value.length : null, reason: args[2] ?? null };
  if (name === "onProcessTimeout") return {};
  return {};
}

function registerVitestClose(context) {
  try {
    if (typeof context?.onClose !== "function") return;
    recordBrowserLifecycle("vitest-close-callback-registered");
    context.onClose(() => recordBrowserLifecycle("vitest-close-callback"));
  } catch (error) {
    recordBrowserLifecycle("vitest-close-registration-error", errorDetails(error));
  }
}

function wrapReporterHook(reporter, name) {
  const original = typeof reporter[name] === "function" ? reporter[name] : undefined;
  try {
    reporter[name] = function (...args) {
      const details = hookDetails(name, args);
      recordBrowserLifecycle(`${name}:start`, details);
      let result;
      try {
        result = original ? Reflect.apply(original, this, args) : undefined;
      } catch (error) {
        recordBrowserLifecycle(`${name}:error`, { ...details, ...errorDetails(error) });
        throw error;
      }
      if (name === "onInit") registerVitestClose(args[0]);
      if (observeCompletion(result, name, details)) return result;
      recordBrowserLifecycle(`${name}:complete`, details);
      return result;
    };
  } catch (error) {
    recordBrowserLifecycle("reporter-hook-wrap-error", { hook: name, ...errorDetails(error) });
  }
}

/** Instrument public Vitest reporter hooks while preserving each hook's behavior. */
export function installBrowserLifecycleReceipts(reporter) {
  if (!reporter || typeof reporter !== "object" || wrappedReporters.has(reporter)) return reporter;
  wrappedReporters.add(reporter);
  for (const name of [
    "onInit",
    "onBrowserInit",
    "onTestRunStart",
    "onTestModuleQueued",
    "onTestModuleCollected",
    "onTestModuleStart",
    "onTestModuleEnd",
    "onTestCaseReady",
    "onTestCaseResult",
    "onTestRunEnd",
    "onProcessTimeout",
  ]) {
    wrapReporterHook(reporter, name);
  }
  return reporter;
}

function wrapProviderMethod(provider, name) {
  const original = provider[name];
  if (typeof original !== "function") return;
  try {
    provider[name] = function (...args) {
      const phase = name === "openPage" ? "provider.openPage" : "provider.close";
      const details = name === "openPage" ? { parallel: args[2]?.parallel ?? null } : {};
      recordBrowserLifecycle(`${phase}:start`, details);
      let result;
      try {
        result = Reflect.apply(original, this, args);
      } catch (error) {
        recordBrowserLifecycle(`${phase}:error`, { ...details, ...errorDetails(error) });
        throw error;
      }
      if (observeCompletion(result, phase, details)) return result;
      recordBrowserLifecycle(`${phase}:complete`, details);
      return result;
    };
  } catch (error) {
    recordBrowserLifecycle("provider-method-wrap-error", { method: name, ...errorDetails(error) });
  }
}

/** Wrap only the public provider factory and lifecycle methods; keep option identity. */
export function observeBrowserProvider(option) {
  if (!option || typeof option !== "object" || wrappedProviders.has(option)) return option;
  wrappedProviders.add(option);
  const factory = option.providerFactory;
  if (typeof factory === "function") {
    try {
      option.providerFactory = function (...args) {
        recordBrowserLifecycle("provider.factory:start");
        let provider;
        try {
          provider = Reflect.apply(factory, this, args);
        } catch (error) {
          recordBrowserLifecycle("provider.factory:error", errorDetails(error));
          throw error;
        }
        const instrument = (value) => {
          if (value && typeof value === "object") {
            wrapProviderMethod(value, "openPage");
            wrapProviderMethod(value, "close");
          }
          return value;
        };
        recordBrowserLifecycle("provider.factory:complete");
        return instrument(provider);
      };
    } catch (error) {
      recordBrowserLifecycle("provider-factory-wrap-error", errorDetails(error));
    }
  }
  if (typeof option.prewarm === "function") {
    const prewarm = option.prewarm;
    try {
      option.prewarm = function (...args) {
        recordBrowserLifecycle("provider.prewarm:dispatch");
        return Reflect.apply(prewarm, this, args);
      };
    } catch (error) {
      recordBrowserLifecycle("provider-prewarm-wrap-error", errorDetails(error));
    }
  }
  return option;
}
