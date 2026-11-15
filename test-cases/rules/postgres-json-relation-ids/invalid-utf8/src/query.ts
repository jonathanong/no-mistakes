// stray ÿ
import { query } from "@example/db";
export const insert = () => query(`INSERT INTO orders (details)
 VALUES (jsonb_build_object('accountId', $1))`);
