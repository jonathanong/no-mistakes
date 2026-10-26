globalThis[`fetch`]("/api/users");
fetch.call(globalThis, "/api/users");
(0, fetch)("/api/users");
const { ["fetch"]: fromLiteral, [`fetch`]: fromTemplate } = globalThis;
fromLiteral("/api/users");
fromTemplate("/api/users");
const key = "setTimeout";
const { [key]: schedule } = globalThis;
schedule(() => {}, 0);
