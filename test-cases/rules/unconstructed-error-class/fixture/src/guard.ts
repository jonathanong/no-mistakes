// Flagged: this is the dead-handler shape. The class is only narrowed with
// `instanceof` and a type guard, so nothing can throw it.
export class InstanceofOnlyError extends Error {}

export function isInstanceofOnly(value: unknown): value is InstanceofOnlyError {
  return value instanceof InstanceofOnlyError;
}
