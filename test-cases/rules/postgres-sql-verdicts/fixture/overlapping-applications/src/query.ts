import { query } from "@example/db";

// The same executor call is evaluated under two named policy applications.
export const rows = query("SELECT * FROM orders");
