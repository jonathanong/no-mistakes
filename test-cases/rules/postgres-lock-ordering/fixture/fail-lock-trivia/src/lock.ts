import { query } from "@example/db";

export function lockRows(ids: string[]) {
  query(`SELECT * FROM t WHERE id = ANY($1) FOR
UPDATE`);
  query(`SELECT * FROM t WHERE id = ANY($1) FOR  UPDATE`);
  query(`SELECT * FROM t WHERE id = ANY($1) FOR /* legal gap */ UPDATE`);
  query(`SELECT * FROM t WHERE id = ANY($1) FOR -- legal gap
UPDATE`);
}
