declare const userId: string;
declare const method: string;

const id = userId;
(globalThis as typeof globalThis).fetch(`/api/${id}`, { method });
const { fetch: request } = globalThis as typeof globalThis;
request(`/api/${id}`, { method });
