import { query } from "@example/db";
const registry = require("./tables");
query(`SELECT id FROM ${registry.tables.users}`);
