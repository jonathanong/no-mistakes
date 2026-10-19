import { query } from "@example/db";
// Lock words in a comment do not establish a parsed locking SELECT.
query("SELECT 1 /* FOR UPDATE */");
query("SELECT * FROM accounts WHERE (id IN (1, 2) FOR UPDATE");
query("SELECT * FROM mystery WHERE id IN (1, 2) ORDER BY id FOR UPDATE");
query("SELECT * FROM accounts WHERE id IN (1, 2) ORDER BY name FOR UPDATE");
