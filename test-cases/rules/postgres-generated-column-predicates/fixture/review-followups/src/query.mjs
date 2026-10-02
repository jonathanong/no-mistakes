import { query } from "@data-stores/psql";
query(`SELECT id FROM orders WHERE created_at > $1`);
