// stray ÿ
import { query } from "@example/db";
query(`INSERT INTO items (id) VALUES ($1), ($2) ON CONFLICT (id) DO NOTHING`);
