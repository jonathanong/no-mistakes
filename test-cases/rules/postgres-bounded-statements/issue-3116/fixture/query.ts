import { query, sql } from "@example/db";
import { columns } from "./fragment";

export function bounded(id: string) {
  const statement = sql`SELECT `;
  statement.append(columns());
  statement.append(sql` FROM image_placements target WHERE target.placement_id = ${id}`);
  return query(statement);
}

export function unbounded() {
  const statement = sql`SELECT `;
  statement.append(columns());
  statement.append(sql` FROM image_placements target`);
  return query(statement);
}
