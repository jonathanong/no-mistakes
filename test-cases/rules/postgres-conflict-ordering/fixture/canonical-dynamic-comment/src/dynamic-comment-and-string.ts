import { query } from "@example/db";

declare const tableName: string;

query(`SELECT *
FROM ${tableName}
-- INSERT INTO items ON CONFLICT (id) DO NOTHING
WHERE note = 'INSERT INTO items ON CONFLICT (id) DO NOTHING'`);
