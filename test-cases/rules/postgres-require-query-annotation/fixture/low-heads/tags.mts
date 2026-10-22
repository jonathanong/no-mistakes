// Low effort deliberately ignores tag identity; only the first literal head matters.
import { read } from "@bench/db";
import { sql as named } from "@bench/tags";
import { sql as subpath } from "@bench/tags/statements";
import { sql as sibling } from "@bench/tags-other";
import { sql as other } from "@other/tags";
import defaultTag from "sql-template-strings";
import { sql as builtinNamed } from "sql-template-strings";
import * as namespace from "@bench/tags";
import type { sql as onlyType } from "@bench/tags";
read(named`/* direct tag */ SELECT ${value}`); // low: true
const tagged = named`/* const tag */ SELECT ${value}`;
read(tagged); // low: true
read(named`/* wrapped tag */ SELECT ${value}`); // low: true
read(subpath`/* subpath */ SELECT ${value}`); // low: true
const subpathConst = subpath`/* subpath const */ SELECT ${value}`;
read(subpathConst); // low: true
read(sibling`/* sibling */ SELECT ${value}`); // low: true
read(other`/* other */ SELECT ${value}`); // low: true
read(defaultTag`/* builtin default */ SELECT ${value}`); // low: true
read(builtinNamed`/* builtin named */ SELECT ${value}`); // low: true
read(namespace.sql`/* namespace */ SELECT ${value}`); // low: true
read(onlyType`/* type import */ SELECT ${value}`); // low: true
read(named`/* open ${value} */ SELECT 1`); // low: false
read(named`${value} /* late head */ SELECT 1`); // low: false
read(named`/* */ /* later */ SELECT 1`); // low: false
read(named`SELECT ${value}`); // low: false
const aliasSource = named`/* alias source */ SELECT ${value}`;
const alias = aliasSource;
read(alias); // low: false
let mutable = named`/* let tag */ SELECT ${value}`;
read(mutable); // low: false
const tagAlias = named;
read(tagAlias`/* alias tag */ SELECT ${value}`); // low: true
const concatenated = named`/* tag plus */ SELECT ${value}` + tail;
read(concatenated); // low: true
read(named`/* direct tag plus */ SELECT ${value}` + tail); // low: true
function shadow(named: unknown) {
  read(named`/* parameter shadow */ SELECT ${value}`); // low: true
}
{
  const named = localTag;
  read(named`/* block shadow */ SELECT ${value}`); // low: true
}
function captureLater() {
  read(laterTagged); // low: true
}
const laterTagged = named`/* later captured const */ SELECT ${value}`;
captureLater();
read("/* plain literal */ SELECT 1"); // low: true

const defaultStatement = defaultTag`/* default const */ SELECT ${value}`;
read(defaultStatement); // low: true
const namespaceStatement = namespace.sql`/* namespace const */ SELECT ${value}`;
read(namespaceStatement); // low: true
read(defaultTag`SELECT ${value}`); // low: false
read(defaultTag`${value} /* late default */ SELECT 1`); // low: false
read(namespace["sql"]`/* computed tag */ SELECT ${value}`); // low: true
read(makeTag()`/* factory tag */ SELECT ${value}`); // low: true
const computedStatement = namespace["sql"]`/* computed const */ SELECT ${value}`;
read(computedStatement); // low: true

// Concatenation keeps the first head; a later tag cannot repair an unknown one.
read((named`/* nested tag plus */ SELECT ${value}` + tail) + tail); // low: true
read(tail + named`/* right tag */ SELECT ${value}`); // low: false
read(named`/* ${value} incomplete */ SELECT 1` + tail); // low: false
const wrappedPlus = (defaultTag`/* wrapped const plus */ SELECT ${value}` + tail) as string;
read(wrappedPlus); // low: true
