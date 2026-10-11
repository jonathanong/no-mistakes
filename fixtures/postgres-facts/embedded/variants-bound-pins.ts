import { query, sql } from "@example/db";
const gate = runtimeFlag;
query(gate ? sql`
  SELECT * FROM public.accounts a
  WHERE a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
  SELECT * FROM public.accounts
  WHERE id IN (SELECT d.id FROM LATERAL (SELECT id) d JOIN orders o ON true LIMIT 1);
` : sql`SELECT 1`);
