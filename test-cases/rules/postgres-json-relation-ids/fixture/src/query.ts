import { query } from "@example/db";
export const insert = () => query(`INSERT INTO orders (details)
 VALUES (jsonb_build_object('accountId', $1))`);
export const update = () => query(`UPDATE orders SET
 details = '{"accountId": "a1"}'::jsonb WHERE id = $1`);
export const good = () => query(`INSERT INTO orders (account_id, details)
 VALUES ($1, jsonb_build_object('accountId', $1))`);
