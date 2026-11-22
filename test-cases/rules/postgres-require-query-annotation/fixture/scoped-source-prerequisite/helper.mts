import { query } from "@app/db";
import { statement } from "./broken.mjs";
export function run() { query(statement()); }
