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
// Proven forms, in source order: A; A; B; C+D; default x7; E+F; G.
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
// A reference that is never consumed (reassignment) keeps the binding proven.
let Reassigned = dynamic(() => import("./a"));
Reassigned = other;
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
