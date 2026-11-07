import { query } from "@example/db";

export function placeholders(n: number) {
  const conditions = [`id = $${n}`, "active = $" + (n + 1)];
  const where = conditions.join(" AND ");
  query(`SELECT id FROM records WHERE ${where}`);
}
