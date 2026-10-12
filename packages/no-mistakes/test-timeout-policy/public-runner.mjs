const constructors = new WeakMap();
/** Compose a final public constructor while retaining direct and derived new.target contracts. */
export function guardPublicRunner(original, maximum, protect) {
  return guardRunner(original, maximum, protect, false);
}

/** Internal adapter for the native SDK browser subclass, whose collection lifecycle is public. */
export function guardCollectionRunner(original, maximum, protect) {
  return guardRunner(original, maximum, protect, true);
}

function guardRunner(original, maximum, protect, defer) {
  let policies = constructors.get(original);
  if (!policies) {
    policies = new WeakMap();
    constructors.set(original, policies);
  }
  let deadlines = policies.get(protect);
  if (!deadlines) {
    deadlines = new Map();
    policies.set(protect, deadlines);
  }
  let adapters = deadlines.get(maximum);
  if (!adapters) {
    adapters = new Map();
    deadlines.set(maximum, adapters);
  }
  const existing = adapters.get(defer);
  if (existing) return existing;
  const methods = new WeakMap();
  const guarded = new Proxy(original, {
    get(target, key, receiver) {
      const direct = receiver === guarded;
      const value = Reflect.get(target, key, direct ? target : receiver);
      if (!direct || typeof value !== "function") return value;
      const descriptor = staticDescriptor(target, key);
      // Callable-valued fields are public values, including Vitest's exported static aliases.
      if (
        !descriptor ||
        typeof descriptor.value !== "function" ||
        descriptor.enumerable === true ||
        (descriptor?.configurable === false && descriptor.writable === false)
      )
        return value;
      const existing = methods.get(value);
      if (existing) return existing;
      const method = new Proxy(value, {
        apply(callable, owner, args) {
          return Reflect.apply(callable, owner === guarded ? target : owner, args);
        },
      });
      methods.set(value, method);
      return method;
    },
    construct(target, args, newTarget) {
      const runner = Reflect.construct(target, args, newTarget === guarded ? target : newTarget);
      if (newTarget === guarded) protect(runner, maximum);
      else if (defer) {
        // super() returns before derived fields initialize. Vitest's public
        // collection lifecycle runs after construction, before registration.
        const beforeCollect = Reflect.get(runner, "onBeforeCollect");
        Object.defineProperty(runner, "onBeforeCollect", {
          configurable: false,
          writable: false,
          value(...paths) {
            protect(runner, maximum);
            return typeof beforeCollect === "function"
              ? Reflect.apply(beforeCollect, runner, paths)
              : undefined;
          },
        });
      }
      return runner;
    },
  });
  adapters.set(defer, guarded);
  return guarded;
}
/** Class-owned descriptors distinguish methods from fields without changing Function APIs. */
function staticDescriptor(target, key) {
  let owner = target;
  while (owner && owner !== Function.prototype) {
    const descriptor = Reflect.getOwnPropertyDescriptor(owner, key);
    if (descriptor) return descriptor;
    owner = Reflect.getPrototypeOf(owner);
  }
  return undefined;
}
