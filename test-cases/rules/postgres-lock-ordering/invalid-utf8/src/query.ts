// stray ÿ
import { query } from "@example/db";
query(`SELECT id FROM t WHERE id = ANY($1) FOR UPDATE`);
