import {read} from "@bench/db";
import * as direct from "./plain.mjs";
const {named} = direct;
read(named()); // low: true
import * as reexport from "./reexport.mjs";
const {make: reexported} = reexport;
read(reexported()); // low: true
import * as alias from "./alias.mjs";
const {make: aliased} = alias;
read(aliased()); // low: true
import * as cycle from "./cyclic-a.mjs";
const {make: cyclic} = cycle;
read(cyclic()); // low: false
import * as typeOnly from "./only-type.mjs";
const {make: typed} = typeOnly;
read(typed()); // low: false
import * as repeated from "./duplicate.mjs";
const {make: duplicate} = repeated;
read(duplicate()); // low: false
import * as absent from "./not-prepared.mjs";
const {make: absentHelper} = absent;
read(absentHelper()); // low: false
import * as unresolved from "./unresolved.mjs";
const {make: unresolvedHelper} = unresolved;
read(unresolvedHelper()); // low: false
import * as broken from "./broken.mjs";
const {make: brokenHelper} = broken;
read(brokenHelper()); // low: false
import * as noExport from "./plain.mjs";
const {nonexistent} = noExport;
read(nonexistent()); // low: false
import * as nestedNamespace from "./namespace-export.mjs";
const {make: namespaceValue} = nestedNamespace;
read(namespaceValue()); // low: false
import * as escaped from "./plain.mjs";
const {named: escapedHelper} = escaped;
opaque(escaped);
read(escapedHelper()); // low: false
import * as shallow from "./short-0.mjs";
const {make: shortHelper} = shallow;
read(shortHelper()); // low: true
import * as deep from "./deep-0.mjs";
const {make: deepHelper} = deep;
read(deepHelper()); // low: false
