import sql from "sql-template-strings";
import { read } from "@example/db";
import { SHARED as fragment } from "./barrel.mts";
import "./mutator.mts";
// Only the string control is immutable; const builder exports may mutate elsewhere.
const query = sql`/* sibling-mutated named import */`;
query.append(fragment);
read(query);
