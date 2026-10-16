import { query, sql } from "@example/db";

query(`/* posts/update */ WITH changed AS (UPDATE posts SET seen = true RETURNING id)
SELECT id FROM changed`);
query(flag ? "/* posts/one */ TABLE posts" : "/* posts/two */ SELECT 2");
query(dynamicSql);
// One conditional alternative has a generated interpolation marker.
query(flag ? sql`/* posts/runtime */ SELECT ${runtimeSql}` : "/* posts/static */ SELECT 1");
// An untrusted interpolated branch stays opaque, rather than inventing alternatives.
query(flag ? `/* posts/unsafe */ SELECT ${runtimeSql}` : "/* posts/other */ SELECT 3");
