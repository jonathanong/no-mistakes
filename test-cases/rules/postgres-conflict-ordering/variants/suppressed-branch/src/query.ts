import { query } from "@example/db";
query(flag
  // no-mistakes-disable-next-line postgres-conflict-ordering
  ? "INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING"
  : "INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING");
