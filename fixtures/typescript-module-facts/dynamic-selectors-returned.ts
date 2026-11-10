declare function resetModules(): void;
declare function use(value: any): void;
declare const key: string;
declare const flag: boolean;

function importWorker() {
  resetModules();
  function inner() {
    return import("./nested");
  }
  return import("./a");
}
const { default: worker } = await importWorker();
worker;

const load = () => {
  resetModules();
  return import("./b");
};
const { A } = await load();
const { B } = await load();

const loadC = function () {
  return import("./c");
};
const { C, D: d } = await loadC();
d;

function memberAccess() {
  return import("./d");
}
const loaded = await memberAccess();
loaded.default;

function spread() {
  return import("./e");
}
const { ...rest } = await spread();
rest;

function passed() {
  return import("./f");
}
use(await passed());

function forwarded() {
  return import("./g");
}
use(forwarded);

function hop() {
  return import("./h");
}
function hop2() {
  return hop();
}
const { default: hopped } = await hop2();
hopped;

function branched() {
  if (flag) return import("./i");
  return import("./j");
}
const { default: branch } = await branched();
branch;

export function exported() {
  return import("./k");
}
const { default: exp } = await exported();
exp;

function* generated() {
  return import("./l");
}
const { default: gen } = await generated();
gen;

let rebound = function () {
  return import("./m");
};
const { default: letBound } = await rebound();
letBound;

function uncalled() {
  return import("./n");
}

function computed() {
  return import("./o");
}
const { [key]: named } = await computed();
named;

function notAwaited() {
  return import("./p");
}
const { default: bare } = notAwaited();
bare;

function aliased() {
  return import("./q");
}
const alias = aliased;
const { default: fromAlias } = await alias();
fromAlias;

function optional() {
  return import("./r");
}
const { default: opt } = await optional?.();
opt;

function written() {
  return import("./s");
}
written = () => import("./t");
const { default: assigned } = await written();
assigned;

function typed() {
  return import("./u");
}
type TypedLoad = typeof typed;
const { default: typedLoad } = await typed();
typedLoad;
