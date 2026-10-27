import { query } from "@db";

query("SELECT * FROM orders");

query("THIS IS NOT SQL");

query("THIS IS NOT SQL");

const sqlText = chooseSql();
query(sqlText);
