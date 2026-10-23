// no-mistakes-disable-file restricted-api-usage
import * as api from "@example/client";
import * as other from "@example/other";
export { paginateEntries as reexp } from "@example/client";
export { "paginate-entries" as stringExport } from "@example/client";
export * from "@example/client";
export * as ns from "@example/client";
const { paginateEntries: d } = api;
d();
const alias = api.paginateEntries;
alias();
const m = await import("@example/client");
m.paginateEntries();
require("@example/client").paginateEntries();
new api.paginateEntries();
api.paginateEntries.call(null);
api.paginateEntries`tag`;
const { ["paginateEntries"]: computed, paginateEntries: fallback = null, ...rest } = api;
let assigned;
({ paginateEntries: assigned } = api);
// Origin and type-only controls must stay silent without a lexical selector.
export type { paginateEntries } from "@example/client";
export { type paginateEntries as T, allowed } from "@example/client";
export type * from "@example/client";
export { paginateEntries } from "@example/other";
export * from "@example/other";
export const unrelated = 1;
const { allowed } = api;
const { paginateEntries: unrelatedD } = other;
function shadow(api: any, require: any) {
  const { paginateEntries } = api;
  api.paginateEntries;
  require("@example/client");
}
import("@example/other");
require("@example/other");
import(moduleName);
require(moduleName);
require();
const { [key]: unknown } = api;
function param({ paginateEntries }: any) {}
const { paginateEntries: empty } = unrelated;
// Whole-namespace aliases must request a static import before the origin is hidden.
const wholeNamespace = api;
let namespaceAssignment;
namespaceAssignment = api;
import(`@example/client`);
require(`@example/client`);
import(`@example/other`);
require(`@example/other`);
