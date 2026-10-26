import sql from "sql-template-strings";
import { read } from "@example/db";
import { FRAGMENT } from "./fragments.mts";
// Absence of a local mutation does not prove the exported object immutable.
const query = sql`/* mutable object */`;
query.append(FRAGMENT);
read(query);
