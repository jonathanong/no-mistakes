// Extra exports are fail-closed shapes. Do not drop them: the snapshot must
// ignore let, calls, functions, destructuring, and type-only names.

export const ORDERS = "orders" as const;

export let mutableTable = "orders";

function tableName(): string {
  return "orders";
}

// A call is a computed initializer, not a string literal.
export const computed = tableName();

// A function export is not a string constant.
export function notAConst(): string {
  return "orders";
}

// A destructured export has no single literal initializer.
export const { owner: renamed } = { owner: "orders" };

// Type-only names are not values, including a specifier marked type.
export type { ORDERS as OrdersType };
export { type computed as ComputedType };

// An unresolved local name has no binding to inline.
export { notDeclared };

// A rename is the same local constant. Re-exports are not local bindings.
// Do not delete them: neither form may invent a second string constant.
export { ORDERS as ordersAlias };
export { "orders" as ordersLabel } from "./tables.mts";
export { computed as computedAlias } from "./tables.mts";
