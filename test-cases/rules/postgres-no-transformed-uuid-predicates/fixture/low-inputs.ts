import { query } from "db";
query("SELECT 1 FROM accounts WHERE i\u0064::text = $1");
declare const value: string;
query(`SELECT 1 FROM accounts WHERE id::text = ${value}`);
function build() { return "SELECT 1 FROM accounts WHERE id::text = $1"; }
query(build());
query("SELECT 1 FROM accounts WHERE id = $1::uuid");
