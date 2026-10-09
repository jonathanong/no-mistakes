import { write } from "./db.mjs";
import { statement } from "./helper.mjs";
export function execute() { return write(statement()); }
