export function guardTimeouts<T extends { extend: (...args: unknown[]) => T }>(base: T): T {
  return base.extend({});
}
