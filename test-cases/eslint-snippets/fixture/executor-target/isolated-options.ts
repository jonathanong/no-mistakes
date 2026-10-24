import { read, type QueryExecutor } from "@example/db";
import type { AliasOptions } from "./scoped-options-contract";
import { importedDirect } from "./options-escape-helpers";

function named(options: AliasOptions) { options.execute("BEGIN"); }
function destructured({ execute }: AliasOptions) { execute("BEGIN"); }
function inline({ query }: { query: QueryExecutor }) { query("BEGIN"); }
// An isolated parser program loses imported contracts; the inline one is explicit.
named({ execute: read });
destructured({ execute: read });
inline({ query: read });
importedDirect({ query: read, limit: 1 });
