import { query } from "@example/db";

// A guarded SELECT proves the replay guard, but the trigger remains independent.
query("INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1)");
query("INSERT INTO audited (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM audited WHERE id = 1)");
// Unsupported expression dependencies cannot be certified as a safe arbiter.
query("INSERT INTO users (email) VALUES ('a') ON CONFLICT ((array_agg(DISTINCT email))) DO UPDATE SET email = 'b'");
// A known target with no trigger has an independently clear trigger obligation.
query("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING");
declare const opaque: string;
query("INSERT INTO items (");
query(`INSERT INTO items (${opaque}`);
query(opaque);
