import { query } from "@example/db";
query(flag
  ? "INSERT INTO items (id) SELECT input.id FROM unnest($2::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING"
  : "INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING");
