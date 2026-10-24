import sql from "sql-template-strings";
import { read } from "@example/db";
import { RENAMED } from "./barrel.mts";
RENAMED.append(' OFFSET 100');
const query = sql`/* renamed mutation */`;
query.append(RENAMED);
read(query);
