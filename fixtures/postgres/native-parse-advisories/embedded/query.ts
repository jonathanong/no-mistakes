import { query } from "@example/db";
query("INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING;");
