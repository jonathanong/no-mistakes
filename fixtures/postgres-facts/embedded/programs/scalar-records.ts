import sql from 'sql-template-strings';
import type { SQLStatement } from 'sql-template-strings';
export function lifecycle(status: string, at: unknown) {
  return {
    aAt: status === 'x' ? at : null,
    absent: undefined,
    enabled: true,
    amount: 2,
    label: 'scalar',
  };
}
export const arrow = (value: unknown) => ({ value });
export function notRecord() { return sql`SELECT 1`; }
export function spread(value: unknown) { return { ...value, own: 1 }; }
export function prototype(value: unknown) { return { __proto__: value, own: 1 }; }
export function duplicate(value: unknown) { return { own: value, own: 1 }; }
export function getter() { return { get own() { return 1; } }; }
export function method() { return { own() { return 1; } }; }
export function builder(value: SQLStatement) { return { own: value }; }
export function defaulted(value: unknown = 'scalar') { return { own: value }; }
export function global() { return { own: missing }; }
export function member(value: { own: unknown }) { return { own: value.own }; }
export function called(value: unknown) { return { own: unknownCall(value) }; }
export function sqlField() { return { own: sql`SELECT 1` }; }
export function nested() { return { own: { nested: 1 } }; }
export function array() { return { own: [1] }; }
export function extra(value: unknown) { const unused = 1; return { own: value }; }
export function badTest(value: unknown) { return { own: missing ? value : null }; }
export function badYes(value: unknown) { return { own: value ? missing : null }; }
export function badNo(value: unknown) { return { own: value ? null : missing }; }
export function badLeft(value: unknown) { return { own: missing === value }; }
export function badRight(value: unknown) { return { own: value === missing }; }
export function unusedDefault(value: unknown = unknownCall(), actual: unknown) { return { own: actual }; }
export function unusedBuilder(value: SQLStatement, actual: unknown) { return { own: actual }; }
export function looseEquality(value: unknown) { return { own: value == 'scalar' }; }
