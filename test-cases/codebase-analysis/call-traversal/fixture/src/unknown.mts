export function unknown() {
  globalThis.setTimeout();
  runner[method]();
  const globalThis = { setTimeout() {} };
}
