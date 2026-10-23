import type { Pool } from "@example/db";
import { writePool } from "../db.mjs";

function named(pool: Pool) {
  pool.query("/* accounts/list */ SELECT id FROM accounts");
}

function unnamed(pool: Pool) {
  pool.query("SELECT id FROM accounts");
}

named(writePool);
unnamed(writePool);
