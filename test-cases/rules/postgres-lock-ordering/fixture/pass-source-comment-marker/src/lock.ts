import { query } from "@example/db";
/* deadlock-safe: unique key proven by caller */
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
