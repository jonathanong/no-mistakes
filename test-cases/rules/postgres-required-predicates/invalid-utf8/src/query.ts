// stray ÿ
import { query } from "@example/db";
query(`SELECT id FROM orders WHERE id = $1`);
