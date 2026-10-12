import { query as execute, read } from "@example/db";

const byId = `/* posts/by-id */ SELECT id FROM posts WHERE id = ${id}`;
execute(byId);
read(`/* posts/list */ WITH selected AS (SELECT id FROM posts) SELECT * FROM selected`);

// This named local is not an imported executor.
const query = (value: string) => value;
query("/* not-executed */ SELECT 1");
