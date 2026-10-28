declare const key: string;
declare const onLoad: any;
declare const onError: any;
declare const other: any;
declare function dynamic(loader: any): any;
declare function use(value: any): any;
declare function f(value: any): any;
declare function retry(loader: any): any;
declare function wrap(loader: any): any;
declare function outer(value: any): any;
// Proven forms, in source order: A; A; B; C+D; default x8; E+F; G.
import("./a").then((m) => m.A);
// A parameter default never applies to a module namespace, so this is proven.
import("./a").then((m = other) => m.A);
import("./a").then(function (m) { return m.B; });
import("./a").then(({ C, D: d }) => d);
dynamic(() => import("./a"));
const Lazy = dynamic(() => import("./a"));
export default dynamic(() => import("./a"));
export const Exported = dynamic(() => import("./a"));
// Read only through a re-export specifier.
const Reexported = dynamic(() => import("./a"));
export { Reexported };
// An expression-bodied arrow's body is the wrapper's caller only when returned.
const blockStatement = () => { dynamic(() => import("./a")); };
// A `typeof` read never consumes the loaded value.
const Typed = dynamic(() => import("./a"));
export type TypedQuery = typeof Typed;
export const kind = typeof Typed;
// The parameter `shadow` is a different binding, so its use leaves the outer one proven.
const shadow = dynamic(() => import("./a"));
export function shadowed(shadow: any) {
  return shadow.then((m: any) => m.X);
}
export async function proven() {
  const ns = await import("./a");
  const { G } = (await import("./a")) as any;
  return [ns.E, ns.F, G];
}
// Everything below must stay unproven and so must not appear in the facts.
import("./a").then((m) => m[key]);
import("./a").then((m) => m?.A);
import("./a").then(onLoad);
import("./a").then((m) => m.A, onError);
import("./a").then(({ ...rest }) => rest);
import("./a").then(({ [key]: v }) => v);
import("./a").then(({}) => 1);
import("./a").then((a, b) => a.A);
import("./a").then(() => 1);
import("./a").then((m) => 1);
import("./a").then((m) => other.A);
import("./a").then((m) => m.a.b);
import("./a").then((m) => { m.A; });
import("./a").then(function* (m) { yield m.A; });
import("./a")?.then((m) => m.A);
import("./a").then?.((m) => m.A);
use(import("./a").then);
use(import("./a"));
import("./a").catch(onError);
const load = () => import("./a");
// The wrapper result is chained or consumed, so it need not be the module default.
retry(() => import("./a")).then((m) => m.X);
outer(wrap(() => import("./a")));
(retry(() => import("./a"))).then((m) => m.X);
(retry(() => import("./a")) as Promise<any>).then((m) => m.X);
(retry(() => import("./a")) satisfies any).then((m) => m.X);
retry(() => import("./a"))!.then((m) => m.X);
(<any>retry(() => import("./a"))).then((m) => m.X);
const assigned = retry(() => import("./a"));
assigned.then((m) => m.X);
const awaitedLater = retry(() => import("./a"));
await awaitedLater;
const passedLater = retry(() => import("./a"));
use(passedLater);
const { x: destructured } = retry(() => import("./a"));
const wrappedInit = retry(() => import("./a")) as Promise<any>;
wrappedInit.then((m) => m.X);
function returned() {
  return retry(() => import("./a"));
}
const arrowBody = () => retry(() => import("./a"));
const parenArrowBody = () => (retry(() => import("./a")) as any);
// An arrow that is the callee, not an argument, is no loader.
(() => import("./a"))();
// Allow-list: any other parent of the wrapper call may unwrap its result.
(retry(() => import("./a")) ?? other).then((m) => m.X);
(0, retry(() => import("./a"))).then((m) => m.X);
const ternary = key ? retry(() => import("./a")) : other;
use(...retry(() => import("./a")));
const objectInit = { k: retry(() => import("./a")) };
const arrayInit = [retry(() => import("./a"))];
class Holder {
  static C = retry(() => import("./a"));
}
let letBinding = retry(() => import("./a"));
var varBinding = retry(() => import("./a"));
// Every reference to a stored wrapper result other than JSX, typeof and export is consumed.
const aliased = retry(() => import("./a"));
const alias = aliased;
const shorthand = retry(() => import("./a"));
use({ shorthand });
const listed = retry(() => import("./a"));
use([listed]);
const logical = retry(() => import("./a"));
use(logical || other);
const conditional = retry(() => import("./a"));
use(conditional ? 1 : 2);
const spread = retry(() => import("./a"));
use(...spread);
let reassigned = retry(() => import("./a"));
reassigned = other;
function returnsStored() {
  const kept = retry(() => import("./a"));
  return kept;
}
const blockLoad = () => { return import("./a"); };
// Exported, so other modules may read any member even though this one reads A.
export const escaped = await import("./a");
escaped.A;
import("./a").then(([first]) => first);
import("./a").then;
export async function unproven() {
  await import("./a");
  await wrap(() => import("./a"));
  const [first] = await import("./a");
  const passed = await import("./a");
  use(passed);
  const unused = await import("./a");
  const optional = await import("./a");
  optional?.X;
  let reassigned = await import("./a");
  reassigned = other;
  return [first, reassigned.A];
}
