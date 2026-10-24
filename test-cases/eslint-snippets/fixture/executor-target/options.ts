import { read } from "@example/db";
import type { QueryExecutor } from "@example/db";
import type { NamedOptions, AliasOptions, ExtendedOptions } from "./options-helpers";
import { named, alias, inherited, renamed, optional, structural, method, unrelated, wrong, plain, union, mapped, rest, overloaded, contextual, mutable, written } from "./options-helpers";
import { evaluated } from "./options-evaluated";
function localNamed(options: NamedOptions) { options.query("SELECT 1"); }
const localAlias = (options: AliasOptions) => options.query("SELECT 1");
function localInherited(options: ExtendedOptions) { options.query("SELECT 1"); }
function localLiteral(options: { query: QueryExecutor }) { options.query("SELECT 1"); }
named({ query: read });
alias({ query: read });
inherited({ query: read });
localNamed({ query: read });
localAlias({ query: read });
localInherited({ query: read });
localLiteral({ query: read });
optional({ query: read });
structural({ query: read });
wrong({ query: read });
union({ query: read });
rest({ query: read });
overloaded({ query: read });
contextual({ query: read });
mutable({ query: read });
written({ query: read });
evaluated({ query: read });
named({ other: read });
named({ ["query"]: read });
// A spread keeps the object mutable even when the executor property is explicit.
named({ ...extra, query: read });
named({ query: read, ...extra });
named({ query: (statement) => read(statement) });
const query = read;
named({ query });
unrelated({ query: read });
named(...[{ query: read }]);
renamed({ query: read });
method({ query: read });
function localUntyped(options) {}
localUntyped({ query: read });
plain({ query: read });
mapped({ query: read });
