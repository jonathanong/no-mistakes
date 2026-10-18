declare const signal: AbortSignal;
declare const timeoutMs: number;
declare const client: { responses: { create: (options?: object) => void } };

createResponse({ signal });
createResponse({ signal, timeoutMs: undefined });
createResponse({ signal, timeoutMs });
client.responses.create({ signal, timeoutMs });
client.responses.create({ timeoutMs });
