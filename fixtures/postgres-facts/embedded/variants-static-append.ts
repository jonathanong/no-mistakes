import SQL from "sql-template-strings";
import otherSql from "@example/other-sql";
import { query } from "@example/db";

query(SQL`SELECT id FROM users WHERE owner = ${owner}`.append(" AND active").append(SQL` AND team = ${team}`));
const local = SQL`SELECT id FROM users`;
local.append(" WHERE active");
query(local);
const initialized = SQL`SELECT id FROM teams`.append(" WHERE active");
query(initialized);
// Fluent depth must not erase the actual append provenance.
query(SQL`SELECT 4`.append(" /* link0 */").append(" /* link1 */").append(" /* link2 */").append(" /* link3 */").append(" /* link4 */").append(" /* link5 */").append(" /* link6 */").append(" /* link7 */").append(" /* link8 */").append(" /* link9 */").append(" /* link10 */").append(" /* link11 */").append(" /* link12 */").append(" /* link13 */").append(" /* link14 */").append(" /* link15 */").append(" /* link16 */").append(" /* link17 */").append(" /* link18 */").append(" /* link19 */"));
query(SQL`SELECT (`.append(" AND"));

// String methods and unknown tags cannot certify actual SQL append variants.
query("SELECT 6".append(" WHERE active"));
query(String.raw`SELECT 7`.append(" WHERE active"));
query(otherSql`SELECT 8`.append(" WHERE active"));
query(SQL`SELECT 9`.append?.(" WHERE active"));
const escaped = SQL`SELECT 10`.append(" WHERE active");
change(escaped);
query(escaped);
const effected = SQL`SELECT 11`.append(" WHERE active");
query(effected, change(effected));

