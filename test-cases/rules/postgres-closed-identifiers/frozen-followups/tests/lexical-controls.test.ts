import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const names = Object.freeze({ vote: "users", post: "posts" });
type names = string;
interface LocalRecord { names: string }
function guarded(key: string) {
  if (!Object.hasOwn(names, key)) throw new Error("unknown");
  return names[key as keyof typeof names];
}
function withVar() {
  { var names = { vote: "ignored" }; }
  return names.vote;
}
function withFunction() {
  function names() { return "ignored"; }
  return names();
}
function withClass() {
  class names { static value = "ignored"; }
  return names.value;
}
const NamedClass = class names { static value = names; };
const ordinary = (names: string) => names;
function withDestructuring({ names }: LocalRecord) { return names; }
function withCatch() {
  try { throw "ignored"; } catch (names) { return names; }
}
export function run(key: string) {
  void [withVar(), withFunction(), withClass(), NamedClass, ordinary("ignored"),
    withDestructuring({ names: "ignored" }), withCatch()];
  return query(rawSql`SELECT id FROM ${guarded(key)} WHERE id = $1`);
}
