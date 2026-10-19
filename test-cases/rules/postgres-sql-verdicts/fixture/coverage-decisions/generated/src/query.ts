import { query } from "@example/db";
query("SELECT id FROM items WHERE created_at > now()");
// Two joined tables expose created_at, so the unqualified owner stays unknown.
query("SELECT i.id FROM items i JOIN other o ON i.id = o.id WHERE created_at > now()");
