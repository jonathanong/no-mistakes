const fetch = "setTimeout";
const { [fetch]: schedule } = globalThis;
schedule(handler, 0);
const { ["fetch"]: literal } = globalThis;
literal("/api/users");
const { [`fetch`]: template } = globalThis;
template("/api/users");
