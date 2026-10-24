import { enqueueEmail } from "@app/jobs";
import { awaitReportedEnqueueAdmission as consume } from "@app/consumer";
import { vi } from "vitest";

const bare = () => { enqueueEmail(1); };
unknownConsumer(bare);
const unknown = () => enqueueEmail(2);
unknownConsumer(unknown);
function unknownDeclared() { return enqueueEmail(3); }
unknownConsumer(unknownDeclared);

const known = () => enqueueEmail(4);
consume(known);
function knownDeclared() { return enqueueEmail(5); }
consume(knownDeclared);
const knownBare = () => { enqueueEmail(6); };
consume(knownBare);
const wrongSlot = () => enqueueEmail(7);
consume(null, wrongSlot);
const mixed = () => enqueueEmail(8);
consume(mixed);
unknownConsumer(mixed);
const mocked = () => enqueueEmail(9);
consume(mocked);
vi.fn(mocked);

// The declaration name is shadowed inside the body, but its outer binding escapes.
function shadowed(shadowed) { return enqueueEmail(10); }
unknownConsumer(shadowed);
function reassigned() { return enqueueEmail(11); }
reassigned = other;
consume(reassigned);
function ordinary() { return enqueueEmail(12); }
ordinary();

// Ternary paths are unproven; direct async and chained returns adopt the promise.
const ternary = (flag) => flag ? enqueueEmail(13) : Promise.resolve();
consume(ternary);
const asyncDirect = async () => enqueueEmail(14);
consume(asyncDirect);
const chained = () => enqueueEmail(15).then(onSuccess);
consume(chained);
const aliasTarget = () => enqueueEmail(16);
const alias = aliasTarget;
consume(alias);
const explicit = () => { void enqueueEmail(17); };
unknownConsumer(explicit);
let mutable = () => enqueueEmail(18);
consume(mutable); // A writable callback binding cannot establish identity.
const aliasMutable = () => enqueueEmail(19);
let mutableAlias = aliasMutable;
consume(mutableAlias);
const wrapped = () => enqueueEmail(20);
consume(wrapped as () => Promise<void>);
const positioned = () => enqueueEmail(21);
consume.call(null, positioned);
const spread = () => enqueueEmail(22);
consume(...args, spread); // The spread obscures this callback's slot.
const boundUnknown = () => enqueueEmail(23);
unknownConsumer(boundUnknown.bind(null));
const boundKnown = () => enqueueEmail(24);
consume(boundKnown.bind(null)); // bind obscures the original callback's contract.
const idle = () => enqueueEmail(25);
void idle;
const holder = { idle };
export { idle, holder };
// Repeated var declarations can make a cycle in the alias graph.
const cyclic = () => enqueueEmail(26);
var first = cyclic;
var second = first;
var first = second;
consume(first);
const boundOrdinary = () => enqueueEmail(27);
boundOrdinary.bind(null)();
const boundLocal = boundOrdinary.bind(null);
boundLocal();
const boundEscaped = () => enqueueEmail(28);
const escaped = boundEscaped.bind(null);
consume(escaped); // The stored bound callback has an unproven adoption contract.
function replaced() { return enqueueEmail(29); }
var replaced = other; // An initializer write also replaces a function declaration.
consume(replaced);
function overloaded(value: string): unknown;
function overloaded(value: number): unknown;
function overloaded(value: string | number) { return enqueueEmail(30); }
consume(overloaded);
const methodOnly = () => enqueueEmail(31);
unknownConsumer(methodOnly.bind); // Passing bind itself does not pass the callback.
// Passing an invocation's promise does not pass the function as a callback.
unknownConsumer(ordinary());
unknownConsumer(boundOrdinary.bind(null)());
const twiceUnknown = () => enqueueEmail(32);
unknownConsumer(twiceUnknown.bind(null).bind(null));
const twiceKnown = () => enqueueEmail(33);
consume(twiceKnown.bind(null).bind(null));
const twiceStored = () => enqueueEmail(34);
const boundTwice = twiceStored.bind(null).bind(null);
consume(boundTwice);
const twiceLocal = () => enqueueEmail(35);
twiceLocal.bind(null).bind(null)();
const chainedLocal = twiceLocal.bind(null).bind(null);
chainedLocal();
unknownConsumer(twiceLocal.bind(null).bind(null)());
