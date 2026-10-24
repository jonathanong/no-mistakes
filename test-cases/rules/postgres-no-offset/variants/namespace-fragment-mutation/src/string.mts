import sql from "sql-template-strings";
import { read } from "@example/db";
import * as ns from "./fragments.mts";
// A primitive string is immutable and must retain namespace append recovery.
const query = sql`/* immutable namespace string */`;
query.append(ns.STRING);
read(query);
