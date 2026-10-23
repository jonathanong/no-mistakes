// no-mistakes-disable-file postgres-require-query-annotation
import sql from "sql-template-strings";
import { read } from "@bench/db";
read(sql`SELECT (`.append(" AND"));
