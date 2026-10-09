import { write } from "@app/db";
import sql from "sql-template-strings";

// An opaque consumer can call a returned callback as well as the first callback.
const nested = unknownConsumer(() => () => write("SELECT 1")); // finding:nested-callback
const aggregate = unknownConsumer(() => [() => write("SELECT 2")]); // finding:aggregate-callback
const annotated = unknownConsumer(() => () => write("/* returned annotation */ SELECT 1")); // known:annotated-returned-callback

// Repeatedly returning the same callback must stop at the analysis depth bound.
function selfReturning() {
  return selfReturning;
}
const recursive = unknownConsumer(selfReturning);

const localMutation = unknownConsumer(() => {
  return () => {
    const statement = sql``;
    const changed = statement.append("SELECT 3");
    write(statement); // finding:returned-callback-mutates-local-builder
  };
});

const captured = sql`/* returned callback capture */ SELECT 1`;
const capturedMutation = unknownConsumer(() => () => unknownMutation(captured));
write(captured); // unanalyzable:returned-callback-capture

