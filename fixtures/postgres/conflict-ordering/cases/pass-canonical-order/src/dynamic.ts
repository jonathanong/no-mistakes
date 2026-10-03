import { query } from "@example/db";

declare const dynamic: string;

const sql = `INSERT INTO items (id) ${dynamic} ON CONFLICT (id) DO NOTHING`;
query(sql);
