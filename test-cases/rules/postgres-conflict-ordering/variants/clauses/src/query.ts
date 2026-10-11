import { query, sql } from "@example/db"; // cspell:ignore targetless
query(sql`${flag ? sql`
  INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT DO NOTHING -- findings: targetless-arbiter
` : sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING`}`);
query(sql`${flag ? sql`
  INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (note) DO NOTHING -- findings: unresolved-arbiter
` : sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING`}`);
query(sql`${flag ? sql`
  /* deadlock-safe */ INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING
` : sql`INSERT INTO items (id) VALUES ($1) ON CONFLICT (id) DO NOTHING`}`);
