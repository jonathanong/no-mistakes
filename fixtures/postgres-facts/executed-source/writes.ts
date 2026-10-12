import { query } from "@example/db";

query(`/* posts/update */ WITH changed AS (UPDATE posts SET seen = true RETURNING id)
SELECT id FROM changed`);
query(flag ? "/* posts/one */ TABLE posts" : "/* posts/two */ SELECT 2");
query(dynamicSql);
