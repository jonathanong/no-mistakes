import { query } from "@example/db";
// Runtime queries see the final schema even when this path sorts before SQL.
export const insertItem = () => query(`INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING`);
