import sql from "sql-template-strings";
import { read } from "@example/db";
import { EXPORT_MUTATED as fragment } from "./fragments.mts";
// Only the string control is immutable; const builder exports may mutate elsewhere.
const query = sql`/* exporter-mutated named import */`;
query.append(fragment);
read(query);
