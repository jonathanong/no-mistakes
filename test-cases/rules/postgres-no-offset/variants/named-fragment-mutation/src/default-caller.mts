import sql from "sql-template-strings";
import { read } from "@example/db";
import fragment from "./fragments.mts";
fragment.append(' OFFSET 100');
const query = sql`/* default mutation */`;
query.append(fragment);
read(query);
