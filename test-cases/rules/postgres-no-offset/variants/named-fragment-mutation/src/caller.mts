import sql from "sql-template-strings";
import { read } from "@example/db";
import { FRAGMENT } from "./fragments.mts";
import "./mutate.mts";
const query = sql`/* imported mutation */`;
query.append(FRAGMENT);
read(query);
