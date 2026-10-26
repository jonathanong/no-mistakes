import { enqueueEmail } from "@app/jobs";
import { awaitReportedEnqueueAdmission as consume } from "@app/consumer";

const all = (x) => enqueueEmail(x);
await Promise.all(items.map(all));
function settled(x) { return enqueueEmail(x); }
await Promise.allSettled(items.map(settled));
const raced = (x) => enqueueEmail(x);
void Promise.race(items.flatMap(raced));
function any(x) { return enqueueEmail(x); }
function collectAny() { return Promise.any(items.flatMap(any)); }
void collectAny();

const retained = (x) => enqueueEmail(x);
const pending = items.map(retained);
await Promise.all(pending);
function retainedFlat(x) { return enqueueEmail(x); }
const pendingFlat = items.flatMap(retainedFlat);
await Promise.allSettled(pendingFlat);

// Every use must have a proven contract, including a configured consumer.
const mixed = (x) => enqueueEmail(x);
consume(mixed);
items.map(mixed);
const mixedUnknown = (x) => enqueueEmail(x);
await Promise.all(items.map(mixedUnknown));
unknownConsumer(mixedUnknown);

const discarded = (x) => enqueueEmail(x);
items.map(discarded);
function floating(x) { return enqueueEmail(x); }
Promise.all(items.flatMap(floating));
const escaped = (x) => enqueueEmail(x);
const escapedPending = items.map(escaped);
sink(escapedPending);
await Promise.all(escapedPending);
const awaitedArray = (x) => enqueueEmail(x);
await items.map(awaitedArray);
const voidArray = (x) => enqueueEmail(x);
void items.flatMap(voidArray);
function returnArray() {
  const callback = (x) => enqueueEmail(x);
  return items.map(callback);
}

// The second argument is thisArg; a preceding spread obscures the callback slot.
const wrongSlot = (x) => enqueueEmail(x);
await Promise.all(items.map(other, wrongSlot));
const spreadSlot = (x) => enqueueEmail(x);
await Promise.all(items.map(...args, spreadSlot));
let mutable = (x) => enqueueEmail(x);
await Promise.all(items.map(mutable));
const bound = (x) => enqueueEmail(x);
await Promise.all(items.map(bound.bind(null)));

// A synchronous mapper throw may strand promises from earlier iterations.
function throwsLater(x) { if (x.bad) throw error; return enqueueEmail(x); }
const thrownPending = items.map(throwsLater);
await Promise.all(thrownPending);
const asyncThrow = async (x) => { if (x.bad) throw error; return enqueueEmail(x); };
const asyncPending = items.flatMap(asyncThrow);
await Promise.all(asyncPending);

// A local or mutated Promise does not establish an aggregate drain.
function shadowedPromise(Promise) {
  const callback = (x) => enqueueEmail(x);
  return Promise.all(items.map(callback));
}

// Keep the inline callback behavior aligned with the named form.
await Promise.all(items.flatMap((x) => enqueueEmail(x)));
items.flatMap((x) => enqueueEmail(x));

// An array of mapped arrays does not contain the individual promises.
const nested = (x) => enqueueEmail(x);
await Promise.all([items.map(nested)]);
await Promise.all([enqueueEmail(1)]);
const spread = (x) => enqueueEmail(x);
await Promise.all([...items.flatMap(spread)]);
await Promise.all([items.map((x) => enqueueEmail(x))]);

// Ordinary direct calls retain the established function-return contract.
const directFloating = (x) => enqueueEmail(x);
await Promise.all(items.map(directFloating));
directFloating(1);
function directHandled(x) { return enqueueEmail(x); }
await Promise.all(items.flatMap(directHandled));
await directHandled(1);
function directOrdinary(x) { return enqueueEmail(x); }
directOrdinary(1);

// flatMap only proves callbacks that return the promise itself.
const arrayReturned = (x) => [enqueueEmail(x)];
await Promise.all(items.flatMap(arrayReturned));
await Promise.all(items.flatMap((x) => [enqueueEmail(x)]));
const nestedWrapped = (x) => enqueueEmail(x);
await Promise.all([items.map(nestedWrapped) as Promise<unknown>[]]);
const resolvedArray = (x) => enqueueEmail(x);
await Promise.resolve(items.map(resolvedArray));

// Recursive collection disposition must not assume its own contract.
const recursive = (x) => {
  if (x) return enqueueEmail(x);
  return Promise.all(items.map(recursive));
};
consume(recursive);
