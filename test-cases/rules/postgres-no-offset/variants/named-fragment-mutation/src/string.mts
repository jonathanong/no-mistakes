import sql from "sql-template-strings";
import { read } from "@example/db";
import { STRING } from "./barrel.mts";
// Primitive strings remain immutable, including through a named barrel export.
const query = sql`/* immutable named string */`;
query.append(STRING);
read(query);
