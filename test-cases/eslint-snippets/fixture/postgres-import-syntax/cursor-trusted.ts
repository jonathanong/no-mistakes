import * as s from "@example/sql";
import { runCursor } from "@example/cursors";

const { sql } = require("@example/sql");

runCursor(s.sql`/* ok */ SELECT 1`);
runCursor(sql`/* ok */ SELECT 1`);
