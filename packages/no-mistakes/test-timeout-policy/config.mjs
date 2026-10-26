/** Positive, finite configured deadlines; this does not cap aggregate run duration. */
const installedConfigs = new WeakMap();
export function assertTestTimeout(value, label, maximum = 30_000) {
  if (typeof maximum !== "number" || !Number.isFinite(maximum) || maximum <= 0)
    throw new RangeError("Timeout maximum must be positive and finite");
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0 || value > maximum)
    throw new RangeError(`${label} must be positive, finite and at most ${maximum} ms`);
  return value;
}
export function assertTimeoutConfig(config, maximum = 30_000) {
  for (const key of ["testTimeout", "hookTimeout"]) {
    if (Reflect.has(config, key)) assertTestTimeout(Reflect.get(config, key), key, maximum);
  }
}
/** Preserve SDK option/property semantics while capturing one validated deadline. */
export function snapshotTimeoutOptions(options, label, maximum = 30_000, nullInherits = false) {
  // Vitest 5.0.5 assignDefined copies enumerable keys with for-in, including
  // inherited ones. Non-enumerable keys stay unread. Own enumerable timeouts are
  // validated here; an inherited timeout is checked when the task timeout resolves.
  if (!Object.prototype.propertyIsEnumerable.call(options, "timeout")) return options;
  const timeout = Reflect.get(options, "timeout");
  if (timeout !== undefined && (timeout !== null || !nullInherits))
    assertTestTimeout(timeout, label, maximum);
  return new Proxy(options, {
    get(target, key) {
      return key === "timeout" ? timeout : Reflect.get(target, key, target);
    },
  });
}
/** Install before collection, composing with the existing runner's config object. */
export function protectTimeoutConfig(config, maximum = 30_000) {
  const existing = installedConfigs.get(config);
  if (existing !== undefined) {
    if (existing !== maximum) throw new Error("Timeout configuration policy maximum cannot change");
    return;
  }
  const values = ["testTimeout", "hookTimeout"].map((key) => ({
    key,
    value: assertTestTimeout(Reflect.get(config, key), key, maximum),
  }));
  for (const entry of values) {
    const key = entry.key;
    let value = entry.value;
    Object.defineProperty(config, key, {
      enumerable: true,
      configurable: false,
      get: () => value,
      set: (next) => {
        value = assertTestTimeout(next, key, maximum);
      },
    });
  }
  installedConfigs.set(config, maximum);
}
