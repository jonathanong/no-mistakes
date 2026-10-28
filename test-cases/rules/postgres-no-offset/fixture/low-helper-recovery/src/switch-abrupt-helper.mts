// The call target is resolved through prepared module imports, not its spelling.
import { read } from '@example/db';
function escape(value: string) { return value; }
export function assertNever(value: string, label?: string): never {
  throw value;
}
export function returnsInstead(value: string) {
  return value;
}
export function effectBeforeThrow(value: string): never {
  throw new Error(read(value));
}
export function escapeBeforeThrow(value: string): never {
  throw new Error(escape(value));
}
export function coercingThrow(value: unknown): never {
  throw new Error(String(value));
}
