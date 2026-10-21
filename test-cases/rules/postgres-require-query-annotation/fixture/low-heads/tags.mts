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
read(sibling`/* sibling */ SELECT ${value}`); // low: false
read(other`/* other */ SELECT ${value}`); // low: false
read(defaultTag`/* builtin default */ SELECT ${value}`); // low: false
read(builtinNamed`/* builtin named */ SELECT ${value}`); // low: false
read(namespace.sql`/* namespace */ SELECT ${value}`); // low: false
read(onlyType`/* type import */ SELECT ${value}`); // low: false
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
read(tagAlias`/* alias tag */ SELECT ${value}`); // low: false
const concatenated = named`/* tag plus */ SELECT ${value}` + tail;
read(concatenated); // low: false
read(named`/* direct tag plus */ SELECT ${value}` + tail); // low: false
function shadow(named: unknown) {
  read(named`/* parameter shadow */ SELECT ${value}`); // low: false
}
{
  const named = localTag;
  read(named`/* block shadow */ SELECT ${value}`); // low: false
}
function captureLater() {
  read(laterTagged); // low: true
}
const laterTagged = named`/* later captured const */ SELECT ${value}`;
captureLater();
read("/* plain literal */ SELECT 1"); // low: true
