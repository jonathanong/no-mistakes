import { query } from "@example/db";
query("SELECT 1");
query("INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING");
query("INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT DO NOTHING");
