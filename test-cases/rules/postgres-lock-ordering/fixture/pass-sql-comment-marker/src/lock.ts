import { query } from "@example/db";
query(`/* deadlock-safe: unique key proven by caller */ SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
