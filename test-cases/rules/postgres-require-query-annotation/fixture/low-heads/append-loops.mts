import { read } from "@bench/db";
import sql from "any-builder";
function counted() {
  const q = sql`/* counted */ SELECT 1`;
  q.append("suffix");
  for (let i = 0; i < 2; i++) { read(q); q.prepend("unsafe"); } // low: false
}
function keys() {
  const q = sql`/* keys */ SELECT 1`;
  q.append("suffix");
  for (const key in object) { read(q); q.prepend("unsafe"); } // low: false
}
function values() {
  const q = sql`/* values */ SELECT 1`;
  q.append("suffix");
  for (const value of values) { read(q); q.prepend("unsafe"); } // low: false
}
function whileLoop() {
  const q = sql`/* while */ SELECT 1`;
  q.append("suffix");
  while (flag) { read(q); q.prepend("unsafe"); } // low: false
}
function doLoop() {
  const q = sql`/* do */ SELECT 1`;
  q.append("suffix");
  do { read(q); q.prepend("unsafe"); } while (flag); // low: false
}
function unchanged() {
  const q = sql`/* unchanged */ SELECT 1`;
  while (flag) { q.append("suffix"); read(q); } // low: true
}
function withoutAppend() {
  const q = sql`/* no append */ SELECT 1`;
  while (flag) { read(q); q.prepend("unsafe"); } // low: false
}
function arrow() {
  const q = sql`/* arrow */ SELECT 1`;
  q.append("suffix");
  const mutate = () => q.prepend("unsafe");
  read(q); // low: false
}
function directString() {
  read("/* string */ SELECT 1".append("suffix")); // low: true
  const q = "/* initialized string */ SELECT 1".append("suffix");
  read(q); // low: true
}
function optional() {
  const q = sql`/* optional member */ SELECT 1`;
  read(q?.append("suffix")); // low: false
}

function deferredClass() {
  const q = sql`/* deferred */ SELECT 1`;
  q.append("suffix");
  class Later { value = read(q); } // low: false
  q.prepend("unsafe");
  new Later();
}

function capturedInitializer() {
  const q = sql`/* initialized capture */ SELECT 1`.append("suffix");
  function execute() { read(q); } // low: false
}
export const initializedExport = sql`/* initialized export */ SELECT 1`.append("suffix");
read(initializedExport); // low: false

function returnedAliases() {
  const chained = sql`/* chained mutation */ SELECT 1`;
  chained.append("suffix").prepend("unsafe");
  read(chained); // low: false
  const escaped = sql`/* returned escape */ SELECT 1`;
  escape(escaped.append("suffix"));
  read(escaped); // low: false
  const aliased = sql`/* returned alias */ SELECT 1`;
  const alias = aliased.append("suffix");
  alias.prepend("unsafe");
  read(aliased); // low: false
  const unknown = sql`/* unknown call */ SELECT 1`;
  consume(unknown.append("suffix").append("more"));
  read(unknown); // low: false
}
