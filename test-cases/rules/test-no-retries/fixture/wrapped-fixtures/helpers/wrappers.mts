function identity<T>(value: T): T {
  return value;
}

export function guardTimeouts<T extends { extend: (...args: unknown[]) => T }>(base: T): T {
  const wrapped = identity(base).extend({});
  return wrapped;
}
