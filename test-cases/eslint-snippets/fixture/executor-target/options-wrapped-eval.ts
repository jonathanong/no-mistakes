import { read, type QueryExecutor } from "@example/db";
import type { Options } from "./options-escape-helpers";

// Type assertions erase, so these are still direct eval calls at runtime.
const inline = ({ query }: { query: QueryExecutor }) => { (eval as any)("leak(query)"); };
const named = (options: Options) => { (eval as any)("leak(options)"); };
inline({ query: read });
named({ query: read, limit: 1 });
