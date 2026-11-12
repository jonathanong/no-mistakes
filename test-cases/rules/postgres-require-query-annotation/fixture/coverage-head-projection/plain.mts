import sql from "sql-template-strings";
export function named() { return "/* named */ SELECT 1"; }
export function missing() { return "SELECT 1"; }
export const namedValue = "/* namedValue */ SELECT 2";
export const missingValue = "SELECT 2";
export function record() { return {text: "/* record */ SELECT 3", values: []}; }
export function missingRecord() { return {text: "SELECT 3", values: []}; }
export function builder() { return sql`/* builder */ SELECT 4`; }
export function plainText() { return "/* plainText */ SELECT 5"; }
