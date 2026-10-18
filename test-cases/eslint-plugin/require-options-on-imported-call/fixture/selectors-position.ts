declare const input: string;
declare const signal: AbortSignal;
declare const client: { responses: { create: (input: string, options?: object) => void } };

createResponse(input, { signal });
createResponse(input);
createResponse({ signal });
client.responses.create(input, { signal });
client.responses.create({ signal });
