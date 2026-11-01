import sql from "sql-template-strings";

export function scoped(userId: string) {
  return sql` AND scope_id = ${userId}`;
}
