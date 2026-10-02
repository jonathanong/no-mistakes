import { query } from "@data-stores/psql";
const sql = `SELECT id
FROM t
OFFSET 3`;
export const first = query(sql);
export const second = query(sql);
