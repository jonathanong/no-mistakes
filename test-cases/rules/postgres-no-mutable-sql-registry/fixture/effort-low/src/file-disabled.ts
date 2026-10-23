// no-mistakes-disable-file postgres-no-mutable-sql-registry
import { query } from "@example/db";
import { tables } from "./tables";
export function hidden() { return query(`SELECT id FROM ${tables.users}`); }
