import { query } from "@example/db";
query(`SELECT * FROM t WHERE id = ANY($1) AND note = '/* deadlock-safe: SQL data */' FOR UPDATE`);
