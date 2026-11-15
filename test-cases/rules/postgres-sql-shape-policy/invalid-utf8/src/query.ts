// stray ÿ
import { query } from "@example/db";
query(`SELECT id FROM items WHERE uuidv7() = id`);
