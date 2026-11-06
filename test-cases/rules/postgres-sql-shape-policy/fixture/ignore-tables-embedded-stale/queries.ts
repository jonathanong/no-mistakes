import { query } from "db";
query(`SELECT id FROM public.items WHERE tenant_id = $1 ORDER BY id LIMIT $2`);
