export function spread(
  signal: AbortSignal,
  rest: object,
  client: { responses: { create: (options?: object) => void } },
) {
  // A later spread does not erase an earlier static key.
  createResponse({ signal, ...rest });
  createResponse({ ...rest, signal });
  client.responses.create({ signal, ...rest });
  // The later static definition wins, including a definite undefined.
  createResponse({ signal, signal: undefined });
  createResponse({ signal, ...rest, signal: undefined });
  createResponse({ ...rest });
  createResponse({ signal: undefined, ...rest });
  createResponse({ signal: undefined, signal: "ready" });
  client.responses.create({ signal: 1, ...rest, signal: undefined });
  client.responses.create({ signal: undefined, ["signal"]: signal });
}
