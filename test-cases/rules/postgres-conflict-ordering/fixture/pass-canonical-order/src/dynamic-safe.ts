import { query } from "@data-stores/psql";

declare const dynamic: string;

/* deadlock-safe: dynamic fragment is a single ordered values expression */
query(`INSERT INTO items (id) ${dynamic} ON CONFLICT (id) DO NOTHING`);
