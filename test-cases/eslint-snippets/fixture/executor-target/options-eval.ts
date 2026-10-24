import { read, type QueryExecutor } from "@example/db";
import type { Options } from "./options-escape-helpers";

// Direct eval can reach a parameter without a visible identifier reference.
const inline = ({ query }: { query: QueryExecutor }) => { eval("leak(query)"); };
const named = (options: Options) => { eval("leak(options)"); };
inline({ query: read });
named({ query: read, limit: 1 });
