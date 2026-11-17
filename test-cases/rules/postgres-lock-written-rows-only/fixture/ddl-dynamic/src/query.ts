import { beginTransaction } from "@example/db";
import { sql } from "@example/sql";
export async function dynamicAlterTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE sql_placeholder_1 IN SHARE MODE`);
  await tx.query(sql`ALTER TABLE ${relation} ADD COLUMN title text`);
}
export async function dynamicCreateTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE sql_placeholder_1 IN SHARE MODE`);
  await tx.query(sql`CREATE TABLE ${relation} (id integer REFERENCES events(id))`);
  await tx.query(`DROP TABLE events`);
}
export async function dynamicPartitionTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE sql_placeholder_1 IN SHARE MODE`);
  await tx.query(sql`ALTER TABLE events ATTACH PARTITION ${relation} FOR VALUES FROM (1) TO (2)`);
}
