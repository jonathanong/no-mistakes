const verb = requestMethod;
const { fetch: load } = globalThis;
load("/api/users", { method: verb });
globalThis["fetch"]("/api/users", { method: verb });
globalThis[`fetch`]("/api/users", { method: verb });
const f = fetch;
f("/api/users", { method: verb });
(0, fetch)("/api/users", { method: verb });
fetch.call(globalThis, "/api/users", { method: verb });
