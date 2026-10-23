import sql from "sql-template-strings";
import { read } from "@bench/db";

read(sql`SELECT (`.append(" AND"));
read(sql`/* named */ SELECT (`.append(" AND"));
const local = sql`SELECT (`;
local.append(" AND");
read(local);
read(sql``.append("SELECT ("));
read(sql`SELECT 1`.append(" FROM records"));
read("SELECT (");

// A raw composition guess cannot authorize skipping an untrusted runtime method.
import otherSql from "@example/other-sql";
read(otherSql`/* stale */ SELECT (`.append(" AND"));

const empty = sql``;
empty.append("SELECT (");
read(empty);
