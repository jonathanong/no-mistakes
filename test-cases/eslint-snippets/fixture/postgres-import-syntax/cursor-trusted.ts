import * as s from "@example/sql";
import { runCursor } from "@example/cursors";
import { sql as namedSql } from "@example/sql";

const { sql } = require("@example/sql");
const tag = namedSql;

runCursor(s.sql`/* ok */ SELECT 1`);
runCursor(sql`/* ok */ SELECT 1`);
runCursor(tag`/* ok */ SELECT 1`);
