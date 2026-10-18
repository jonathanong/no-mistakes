// no-mistakes-disable-file require-options-on-imported-call -- legacy client reviewed
declare const client: { responses: { create: (options?: object) => void } };

createResponse();
client.responses.create();
