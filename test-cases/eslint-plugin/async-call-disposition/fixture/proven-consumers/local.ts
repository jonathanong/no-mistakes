import { enqueueEmail } from "@app/jobs";

async function consume(callback: () => Promise<unknown>) {
  let result;
  result = await callback();
  return result;
}
const accepted = () => enqueueEmail(1);
await consume(accepted);
function declared() { return enqueueEmail(2); }
await consume(declared);
const consumeConst = async (callback: () => Promise<unknown>) => await callback();
const constConsumer = () => enqueueEmail(3);
await consumeConst(constConsumer);

const unknown = () => enqueueEmail(4); // Unknown consumers still report.
unknownConsumer(unknown);
function floating(callback: () => Promise<unknown>) { callback(); }
const unawaited = () => enqueueEmail(5);
floating(unawaited);
async function escaping(callback: () => Promise<unknown>) {
  await callback();
  unknownConsumer(callback);
}
const escaped = () => enqueueEmail(6);
await escaping(escaped);
const mixed = () => enqueueEmail(7);
await consume(mixed);
unknownConsumer(mixed);
let mutableConsumer = consume;
const mutable = () => enqueueEmail(8);
await mutableConsumer(mutable);
mutableConsumer = floating;
async function overwritten(callback: () => Promise<unknown>) {
  callback = async () => undefined;
  await callback();
}
const replaced = () => enqueueEmail(9);
await overwritten(replaced);
function deferred(callback: () => Promise<unknown>) {
  return async () => await callback();
}
const nested = () => enqueueEmail(10);
deferred(nested);
declare function unknownConsumer(value: unknown): void;
async function evaluated(callback: () => Promise<unknown>) {
  eval("callback = async () => undefined");
  await callback();
}
const dynamic = () => enqueueEmail(11);
await evaluated(dynamic);
async function argumentMutation(callback: () => Promise<unknown>) {
  arguments[0] = async () => undefined;
  await callback();
}
const argumentsAlias = () => enqueueEmail(12);
await argumentMutation(argumentsAlias);
const inspected = () => enqueueEmail(13);
void typeof inspected; // An unrelated non-argument read does not invalidate adoption.
await consume(inspected);
async function evaluatedDefault(callback: () => Promise<unknown>, result = eval("callback = async () => undefined")) {
  await callback();
}
const defaultEvaluation = () => enqueueEmail(14);
await evaluatedDefault(defaultEvaluation);
