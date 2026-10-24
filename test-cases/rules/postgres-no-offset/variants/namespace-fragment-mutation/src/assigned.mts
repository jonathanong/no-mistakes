import sql from "sql-template-strings";
import { read } from "@example/db";
import * as ns from "./fragments.mts";
// Const fixes the binding, but the exported builder still accepts mutations.
ns.FRAGMENT.append(' OFFSET 100');
const query = sql`/* assigned namespace */`;
query.append(ns.FRAGMENT);
read(query);
