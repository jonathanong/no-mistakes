import { read, write, type SessionExecutor } from "@example/db";
import { write as otherWrite } from "@example/other";
import * as db from "@example/db";

function typedOption(options: { query: SessionExecutor }) {
  return options.query("SELECT 1");
}
function forward(statement: unknown, options: unknown) {
  return write(statement, options);
}

export function noStatement(query: SessionExecutor) {
  return write({ query });
}
export function undefinedStatement(query: SessionExecutor) {
  return write(undefined, { query });
}
export function hiddenStatement(query: SessionExecutor, statement: unknown) {
  return write(statement, { query });
}
export function laterStatement(query: SessionExecutor) {
  return write({ query }, "SELECT 1");
}
export function composedStatement(query: SessionExecutor, fragment: string) {
  return write("SELECT " + fragment, { query });
}
export function spreadOption(query: SessionExecutor, label: { id: number }) {
  return typedOption({ ...label, query });
}
export function helperWithSql(query: SessionExecutor) {
  return forward("SELECT 1", { query });
}
export function directSpread(query: SessionExecutor, label: { id: number }) {
  return write("SELECT 1", { ...label, query });
}
export function nestedOption(query: SessionExecutor) {
  return write("SELECT 1", { session: { query } });
}
export function valueArray(query: SessionExecutor) {
  return write("SELECT 1", [query]);
}
export function computedOption(query: SessionExecutor, key: string) {
  return write("SELECT 1", { [key]: query });
}
export function callbackOption() {
  return write("SELECT 1", { query: (statement) => read(statement) });
}
export function namedCallbackOption() {
  const forwarder = (statement: unknown) => read(statement);
  return write("SELECT 1", { query: forwarder });
}
export function callbackBesideOptions(query: SessionExecutor) {
  return write("SELECT 1", { query }, (statement: unknown) => read(statement));
}
export function unrelated(query: SessionExecutor) {
  return otherWrite("SELECT 1", { query });
}
export function shadowed(
  write: (statement: unknown, options: unknown) => unknown,
  query: SessionExecutor,
) {
  return write("SELECT 1", { query });
}
export function reassigned(query: SessionExecutor) {
  let execute = write;
  execute = otherWrite;
  return execute("SELECT 1", { query });
}
export function dynamicCallee(query: SessionExecutor, key: "read" | "write") {
  return db[key]("SELECT 1", { query });
}
export function optionalCallee(query: SessionExecutor) {
  return write?.("SELECT 1", { query });
}
export function optionalReceiver(client: { query: SessionExecutor }, query: SessionExecutor) {
  return client?.query("SELECT 1", { query });
}

class Runner {
  constructor(statement: unknown, options: unknown) {}
}
export function constructorOptions(query: SessionExecutor) {
  return new Runner("SELECT 1", { query });
}
