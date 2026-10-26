import { read, type QueryExecutor } from "@example/db";
function untyped(exec) {}
function rest(...exec: QueryExecutor[]) {}
function overloaded(exec: QueryExecutor): void;
function overloaded(exec: unknown): void {}
function run(exec: QueryExecutor) {}
let mutable = (exec: QueryExecutor) => {};
untyped(read);
rest(read);
overloaded(read);
mutable(read);
missing(read);
run(...other, read);
run([read]);
run({ other: read });
run({ [key]: read });
run(read, read);
new run(read);
function object({ query }: { query: QueryExecutor }) {}
object({ nested: { query: read } });
object([{ query: read }]);
function shadowed() {
  type QueryExecutor = (statement: string) => unknown;
  function local(exec: QueryExecutor) {}
  local(read);
}
function objectWrong({ other }: { query: QueryExecutor; other: unknown }) {}
objectWrong({ query: read });
function objectUntyped({ query }) {}
objectUntyped({ query: read });
function objectNested({ query: { exec } }: { query: QueryExecutor }) {}
objectNested({ query: read });
const changed = (exec: QueryExecutor) => {};
changed = other;
changed(read);
function reassigned(exec: QueryExecutor) {}
reassigned = other;
reassigned(read);
// A literal spread still exposes its executor value, but never proves the slot.
run(...[read]);
