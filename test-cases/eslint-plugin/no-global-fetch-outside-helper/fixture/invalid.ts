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
