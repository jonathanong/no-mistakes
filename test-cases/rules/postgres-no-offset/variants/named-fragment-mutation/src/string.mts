import sql from "sql-template-strings";
import { read } from "@example/db";
import { STRING as fragment } from "./fragments.mts";
// Only the string control is immutable; const builder exports may mutate elsewhere.
const query = sql`/* string named import */`;
query.append(fragment);
read(query);
