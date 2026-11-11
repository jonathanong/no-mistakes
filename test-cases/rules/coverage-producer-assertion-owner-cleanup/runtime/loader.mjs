const stubs = new Map([
  ["fixture-logger", "./stubs/logger.mjs"],
  ["@sentry/node", "./stubs/sentry.mjs"],
  ["node:assert", "./stubs/assertion.mjs"],
]);

export function resolve(specifier, context, nextResolve) {
  const stub = stubs.get(specifier);
  return stub
    ? { url: new URL(stub, import.meta.url).href, shortCircuit: true }
    : nextResolve(specifier, context);
}
