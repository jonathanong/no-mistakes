import { query } from "@example/db";
import { tables } from "./tables";
query(`SELECT id FROM ${tables.users}`);
