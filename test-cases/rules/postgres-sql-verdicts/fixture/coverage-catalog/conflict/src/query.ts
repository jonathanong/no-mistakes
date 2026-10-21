import { query } from "@example/db";

// A static but invalid conflict statement must honor fail versus ignore.
query("INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING");
