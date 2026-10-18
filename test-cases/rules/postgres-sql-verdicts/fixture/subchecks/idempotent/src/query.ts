import { query } from "@example/db";
// Each call keeps sibling obligation outcomes independent.
query("INSERT INTO items (id) VALUES (1)");
query("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = 2");
query("INSERT INTO items (id, note) VALUES (1, 'x') ON CONFLICT (id) DO UPDATE SET note = items.note || 'x'");
query("INSERT INTO items (id, note) VALUES (1, 'x') ON CONFLICT (id) DO UPDATE SET note = random()");
query("INSERT INTO items (id, name) VALUES (1, 'x') ON CONFLICT (name_key) DO UPDATE SET name = EXCLUDED.name");
query("INSERT INTO audited (id) VALUES (1) ON CONFLICT DO NOTHING");
query("INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING");
query("INSERT INTO missing (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id");
