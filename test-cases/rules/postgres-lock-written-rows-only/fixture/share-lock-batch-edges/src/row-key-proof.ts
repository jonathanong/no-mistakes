import { beginTransaction } from "@example/db";
import { sql } from "@example/sql";

export async function castUpdateLockDoesNotProveSameRow(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1::integer FOR UPDATE`, [id]);
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1`, [id]);
}

export async function castWriteKeyRemainsUnresolved(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM jobs WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE jobs SET state = 'done' WHERE id = $1::integer`, [id]);
}

export async function dynamicUpdateLockDoesNotProveRelation(table: string) {
  await using query = await beginTransaction();
  await query(sql`SELECT 1 FROM ${table} AS target WHERE target.id = 1 FOR UPDATE OF target`);
  await query(`SELECT 1 FROM jobs WHERE id = 1 FOR SHARE`);
  await query(`UPDATE jobs SET state = 'done' WHERE id = 1`);
}
