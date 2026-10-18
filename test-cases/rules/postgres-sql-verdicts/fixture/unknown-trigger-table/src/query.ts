import { query } from "@example/db";

// Neither guard proves trigger replay safety without the target table catalog.
query("INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1)");
query("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING");
