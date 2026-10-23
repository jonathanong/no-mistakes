import * as api from "@example/client";
// no-mistakes-disable-next-line restricted-api-usage
export { paginateEntries } from "@example/client";
export * from "@example/client"; // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
export * as ns from "@example/client";
const { paginateEntries: d } = api; // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
const alias = api.paginateEntries;
new api.paginateEntries(); // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
api.paginateEntries.call(null);
api.paginateEntries`tag`; // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
import("@example/client");
require("@example/client"); // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
const wholeNamespace = api;
let namespaceAssignment;
namespaceAssignment = api; // no-mistakes-disable-line restricted-api-usage
// no-mistakes-disable-next-line restricted-api-usage
const { ...copiedNamespace } = api;
const { ...copiedAgain } = api; // no-mistakes-disable-line restricted-api-usage
