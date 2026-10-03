import { query } from "@example/db";
const sql = `SELECT id
FROM t
OFFSET 3`;
export const first = query(sql);
export const second = query(sql);
