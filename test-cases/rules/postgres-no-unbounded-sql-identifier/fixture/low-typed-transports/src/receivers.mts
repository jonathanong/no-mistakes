import type { QueryExecutor } from '@example/db';
import type { QueryExecutor as Unconfigured } from '@other/db';
declare const value: string;
export function typed(run: QueryExecutor) {
  opaque(run);
  run('SELECT * FROM accounts WHERE id::text = $1'); // UUID_CALLEE
}
export function untyped(run) { opaque(run); }
export function nonExecutor(run: unknown) { opaque(run); }
declare function opaque(value: unknown): void;
export function wrongPosition(value: unknown, run: QueryExecutor) { opaque(value); }
export function otherType(run: Unconfigured) { opaque(run); }
export function mixed(proven: QueryExecutor, opaqueValue: unknown) { opaque(opaqueValue); }
