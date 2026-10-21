const fetch = "setTimeout";
const { [fetch]: schedule } = globalThis;
schedule(handler, 0);
const { ["fetch"]: literal } = globalThis;
literal(`/api/${id}`);
