import sql from "sql-template-strings";
export const TABLE = "orders" as const;
export const QUERY = `SELECT id FROM orders WHERE account_id = $1`;
export const FRAGMENT = sql` AND account_id = $1`;
export const MISSING = "SELECT id FROM orders WHERE true";
export let mutable = "orders";
export const dynamic = tableName();
export function tableName() { return "orders"; }
export const object = { TABLE: "orders" };
export default "orders";
