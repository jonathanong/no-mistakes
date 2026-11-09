import { query, sql } from "db";
const unicode = "🦀"; query("SELECT DISTINCT id FROM posts"); query("SELECT DISTINCT id FROM posts");
query(`SELECT DISTINCT id FROM posts WHERE author_id = ${author}`); query(sql`SELECT DISTINCT id FROM posts WHERE author_id = ${author}`);
