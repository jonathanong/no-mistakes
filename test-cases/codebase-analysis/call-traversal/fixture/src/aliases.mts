import { importedTarget as imported } from "./alias-target.mts";
import * as targets from "./alias-target.mts";
import { importedTarget as propertyImport } from "./alias-target.mts";
import { hidden } from "./private-target.mts";
import { remote } from "unresolved-runtime-library";
import { missingReexport } from "./unresolved-local-reexport.mts";
import { importedExternal } from "./external-imported-reexport.mts";
import { absentThroughStar } from "./empty-star-barrel.mts";
import defaultThroughIdentifier from "./reexport-default.mts";
import defaultThroughStar from "./star-default.mts";
import { shared as ambiguous } from "./ambiguous-barrel.mts";
import { missing as cycle } from "./star-cycle-a.mts";
import {
  exportedAlias,
  default as defaultAlias,
  localExportAlias,
} from "./exported-local-aliases.mts";
import { collision } from "./mixed-star-barrel.mts";
import { externalCollision } from "./external-star-barrel.mts";
import { mock as externalMock } from "./external-named-barrel.mts";

const first = imported;
const second = (first as typeof first);
const mutable = imported;
let mutableDeclaration = imported;
const cycleA = cycleB;
const cycleB = cycleA;
const moduleAlias = imported;

first();
second?.();
defaultThroughIdentifier();
defaultThroughStar();
ambiguous();
cycle();
mutable = () => {};
mutable();
mutableDeclaration();
cycleA();
exportedAlias();
defaultAlias();
localExportAlias();
collision();
externalCollision();
externalMock();
targets.importedTarget();
targets.default();
targets.deep.member();
propertyImport.call();
hidden();
remote();
missingReexport();
importedExternal();
absentThroughStar();

globalThis.setTimeout(() => {}, 1);
window.setTimeout(() => {}, 1);
self.setTimeout(() => {}, 1);
global.setTimeout(() => {}, 1);

function shadowed(window: { setTimeout(): void }) {
  window.setTimeout();
}

function sameLineCalls() { setTimeout(() => {}, 1); clearTimeout(0); }

(() => imported())();

export function callsModuleAlias() {
  moduleAlias();
}
