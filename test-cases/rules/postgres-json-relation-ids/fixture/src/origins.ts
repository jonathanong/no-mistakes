import { query } from "@example/db";
const statement = `INSERT INTO orders (details)
 VALUES (jsonb_build_object('accountId', $1))`;
export const shared = () => query(statement);
export const cooked = () => query("UPDATE orders SET\n details = '{\"accountId\": \"a1\"}'::jsonb");
