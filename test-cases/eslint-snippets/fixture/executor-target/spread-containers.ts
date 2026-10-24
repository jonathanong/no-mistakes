import { read, type QueryExecutor } from "@example/db";
function take({ query }: { query: QueryExecutor }) { query("SELECT 1"); }
take({ query: read });
// A spread anywhere prevents proving a fresh object's executor slot.
take({ query: read, ...rest });
take({ ...rest, query: read });
take({ ...{ query: read } });
take({ ...read });
take([{ ...{ query: read } }]);
take({ query: read, nested: { query: read } });
// A nested forwarder remains an adapter even inside a spread.
take({ ...{ query: (statement) => read(statement) } });
