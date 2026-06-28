fetch("/api/bare");
globalThis.fetch("/api/global");
window.fetch("/api/window");
self["fetch"]("/api/self");

const request = fetch;
request("/api/alias");

const windowRequest = window.fetch;
windowRequest("/api/window-alias");

const { fetch: selfFetch } = self;
selfFetch("/api/self-alias");

const { fetch: globalFetch } = globalThis;
const nestedFetch = globalFetch;
nestedFetch("/api/nested-alias");

(fetch as typeof fetch)("/api/cast");
(fetch!)("/api/non-null");
fetch<Response>("/api/instantiation");
(globalThis as typeof globalThis).fetch("/api/root-cast");
(globalThis.fetch as typeof fetch)("/api/member-cast");
globalThis.fetch?.("/api/optional");

const satisfiesFetch = fetch satisfies typeof fetch;
satisfiesFetch("/api/satisfies");
