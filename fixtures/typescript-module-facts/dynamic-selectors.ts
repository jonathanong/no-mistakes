declare const key: string;
declare const onLoad: any;
declare const onError: any;
declare const other: any;
declare function dynamic(loader: any): any;
declare function use(value: any): any;
declare function f(value: any): any;
// Proven forms, in source order: A; A; B; C+D; default; E+F; G.
import("./a").then((m) => m.A);
// A parameter default never applies to a module namespace, so this is proven.
import("./a").then((m = other) => m.A);
import("./a").then(function (m) { return m.B; });
import("./a").then(({ C, D: d }) => d);
dynamic(() => import("./a"));
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
const blockLoad = () => { return import("./a"); };
// Exported, so other modules may read any member even though this one reads A.
export const escaped = await import("./a");
escaped.A;
import("./a").then(([first]) => first);
import("./a").then;
export async function unproven() {
  await import("./a");
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
