declare const client: { responses: { create: (options?: object) => void } };

export function suppressed() {
  // no-mistakes-disable-next-line require-options-on-imported-call -- wrapper sets signal
  createResponse();
  createResponse(); // no-mistakes-disable-line require-options-on-imported-call -- reviewed call
  client.responses.create(); // no-mistakes-disable-line no-mistakes/require-options-on-imported-call -- reviewed shape
  // eslint-disable-next-line no-mistakes/require-options-on-imported-call -- native suppression still applies
  createResponse();
  createResponse();
  client.responses.create();
  const note = "no-mistakes-disable-line require-options-on-imported-call";
  createResponse(note);
}
