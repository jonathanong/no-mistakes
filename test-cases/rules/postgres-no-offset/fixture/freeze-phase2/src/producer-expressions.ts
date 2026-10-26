import { beginTransaction } from "@example/db";
import type { QueryExecutor } from "@example/executor-types";
declare const statement: string;
declare const query: QueryExecutor;
declare const flag: boolean;
declare function unconfigured(): unknown;
declare function opaque(value: unknown): unknown;
declare function receiverFrom(value: unknown): { run: unknown };

// Function identity and returned receiver provenance survive named expressions/aliases.
const produce = function inner(): QueryExecutor {
  return unconfigured() as QueryExecutor;
}; // NON_SQL
const factoryAlias = produce; // NON_SQL
const produced = factoryAlias(); // NON_SQL
produced(statement); // ADVISE_NAMED_FACTORY
const makeRecord = function namedRecord() {
  return { run: query, count: 1 };
}; // NON_SQL
const recordAlias = makeRecord; // NON_SQL
const made = recordAlias(); // NON_SQL
made.run(statement); // ADVISE_RETURNED_RECORD
made.count.toString(); // NON_SQL
const producers = { chosen: produce, unrelated: () => 0 }; // NON_SQL
const nestedFactory = producers.chosen; // NON_SQL
const nested = nestedFactory(); // NON_SQL
nested(statement); // ADVISE_NESTED_FACTORY
// Direct dot and indexed producer calls retain the same negative return provenance.
const directProduced = producers.chosen(); // NON_SQL
directProduced(statement); // ADVISE_DIRECT_PRODUCER
const indexedProduced = producers["chosen"](); // NON_SQL
indexedProduced(statement); // ADVISE_INDEXED_PRODUCER
const directlyDynamic = producers[statement](); // ADVISE_DIRECT_DYNAMIC_FACTORY
directlyDynamic(statement); // ADVISE_DIRECT_DYNAMIC_RECEIVER
const ignoredFactory = producers.unrelated; // NON_SQL
ignoredFactory(); // NON_SQL
// A dynamic producer choice cannot erase the subsequent executor invocation.
const dynamicFactory = producers[statement]; // NON_SQL
const dynamicReceiver = dynamicFactory(); // ADVISE_DYNAMIC_FACTORY
dynamicReceiver(statement); // ADVISE_DYNAMIC_RECEIVER
async function awaited(): Promise<QueryExecutor> {
  return await produce();
} // NON_SQL
async function consumeAwaited() {
  const receiver = await awaited(); // NON_SQL
  receiver(statement); // ADVISE_AWAITED
}
function conditional() {
  return flag ? produce() : query;
} // NON_SQL
const conditionalReceiver = conditional(); // NON_SQL
conditionalReceiver(statement); // ADVISE_CONDITIONAL_RETURN
let reassignedFactory = function original() {
  return query;
}; // NON_SQL
reassignedFactory = function replacement() {
  return query;
}; // NON_SQL
// A reassigned factory is an opaque invocation; its returned receiver remains uncertain.
const reassigned = reassignedFactory(); // ADVISE_REASSIGNED_FACTORY
reassigned(statement); // ADVISE_REASSIGNED_FACTORY
// Store updates broaden the carrier rather than preserving stale property proofs.
const carrier = { run: () => 0 }; // NON_SQL
carrier.run = query; // NON_SQL
carrier.run(statement); // ADVISE_STORE

// Writes through a carrier alias must invalidate both aliases without blaming transport.
const carrierAlias = carrier; // NON_SQL
carrierAlias.run = query; // NON_SQL
carrierAlias.run(statement); // ADVISE_ALIAS_STORE
// A unary conversion is scalar data even when its source is an executor value.
const scalarData = { count: 0 }; // NON_SQL
scalarData.count = +query; // NON_SQL

// Suppressing a write receiver must not suppress executor uses in the key or RHS.
carrierAlias[query(statement)] = 0; // ADVISE_COMPUTED_KEY_SQL
carrierAlias.run = query(statement); // ADVISE_RHS_SQL
carrierAlias.run = opaque(query); // ADVISE_RHS_OPAQUE
receiverFrom(query(statement)).run = 0; // ADVISE_RECEIVER_CALL_SQL

// A possible executor call is not a definite factory-only transport.
const mixedCallee = flag ? query : beginTransaction; // NON_SQL
const mixedResult = mixedCallee(statement); // ADVISE_MIXED_CALLEE
mixedResult(statement); // ADVISE_MIXED_RETURNED_RECEIVER

// Optional invocation still executes the selected executor when present.
query?.(statement); // ADVISE_OPTIONAL_QUERY
