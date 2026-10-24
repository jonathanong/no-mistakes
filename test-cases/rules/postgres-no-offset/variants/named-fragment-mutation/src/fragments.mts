import sql from "sql-template-strings";
export const UNMODIFIED = sql`SELECT id FROM orders`;
export const FRAGMENT = sql`SELECT id FROM orders`;
export const EXPORT_MUTATED = sql`SELECT id FROM orders`;
export const STRING = "SELECT id FROM orders";
// Const fixes the binding, not the builder contents, even in its declaring module.
EXPORT_MUTATED.append(" OFFSET 100");
