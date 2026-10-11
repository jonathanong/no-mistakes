import { used } from "./target";
import * as api from "./target";
used(1);
api.used("namespace");
const alias = used;
alias(true);
const nsAlias = api.used;
nsAlias({});
(api["used"])(null);
// Same spelling, a different lexical binding: these must be excluded.
function shadowed(used: () => void) { used(); }
function shadowedNamespace(api: { used(): void }) { api.used(); }
function shadowedAlias(alias: () => void) { alias(); }
function sibling() {
  const used = () => {};
  used();
}
const unrelated = { used() {} };
unrelated.used();
import { forwarded as renamed } from "./barrel";
renamed([1]); (0, used)(2);
used?.(3);
import type { used as TypeOnly } from "./target";
function onlyType(TypeOnly: () => void) { TypeOnly(); }
let replaced = used;
replaced = () => {};
replaced(4);
import { viaChain } from "./chain";
viaChain(5);
import { used as own } from "./star-shadow";
own("not the original");
import { api as bundled } from "./namespace-barrel";
bundled.used(6);
