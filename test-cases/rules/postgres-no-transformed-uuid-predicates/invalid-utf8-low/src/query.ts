// stray ÿ
import { query } from "@example/db";
query(`SELECT 1 FROM accounts WHERE lower(email) = $1`);
