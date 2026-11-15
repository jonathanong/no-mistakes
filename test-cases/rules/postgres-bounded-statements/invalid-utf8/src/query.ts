// stray ÿ
import { query } from "@example/db";
query(`SELECT note FROM orders WHERE note = $1`);
