import { query } from "@data-stores/psql";

declare const dynamic: string;

const sql = `INSERT INTO items (id) ${dynamic} ON CONFLICT (id) DO NOTHING`;
query(sql);
