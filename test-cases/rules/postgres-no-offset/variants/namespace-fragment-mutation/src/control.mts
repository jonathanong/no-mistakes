import sql from "sql-template-strings";
import { read } from "@example/db";
import * as ns from "./fragments.mts";
// Even without a local mutation, another importer can mutate the shared builder.
const query = sql`/* unchanged namespace fragment */`;
query.append(ns.FRAGMENT);
read(query);
