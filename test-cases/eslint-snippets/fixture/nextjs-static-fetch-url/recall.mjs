const id = userId;
globalThis["fetch"](`/api/${id}`);
const f = fetch;
f(`/api/${id}`);
fetch.call(globalThis, `/api/${id}`);
const urls = [`/api/${id}`];
fetch.apply(null, urls);
