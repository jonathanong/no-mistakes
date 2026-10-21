import { query } from "@example/db";
import { ns } from "./index";
query(`SELECT id FROM ${ns.tables.users}`);
