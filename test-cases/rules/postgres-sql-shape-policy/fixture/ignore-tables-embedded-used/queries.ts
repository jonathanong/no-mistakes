import { query } from "db";
query(`SELECT id FROM public.items ORDER BY id LIMIT $1`);
