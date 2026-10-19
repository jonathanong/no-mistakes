// Building the table map by assignment leaves the field read unresolved.
import { query } from "@example/db";

export const tables = {};
tables.users = "app.users";
tables.orders = "app.orders";

query(`SELECT id FROM ${tables.users} WHERE active`);
