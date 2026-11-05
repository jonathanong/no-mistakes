import { query } from "@example/db";
const note = '/* deadlock-safe: documentation string */';
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
