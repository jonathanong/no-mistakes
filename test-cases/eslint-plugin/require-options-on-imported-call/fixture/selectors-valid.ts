declare function getClient(): { responses: { create: (options?: object) => void } };
declare const dynamic: string;

export function accepted(
  signal: AbortSignal,
  rest: object,
  opts: object,
  client: {
    responses: { create: (options?: object) => void };
  },
  agent: { responses: { create: (options?: object) => void } },
) {
  createResponse({ signal });
  createResponse?.({ signal });
  createResponse({ signal, ...rest });
  createResponse({ ...opts, signal });
  createResponse({ ["signal"]: signal });
  createResponse({ [`signal`]: signal });
  createResponse({ signal: undefined, signal });
  createResponse!({ signal });
  (createResponse as Function)({ signal });
  globalThis.createResponse({ signal });
  globalThis?.createResponse?.({ signal });
  window.createResponse({ signal });
  self.createResponse({ signal });
  global.createResponse({ signal });
  client.responses.create({ signal });
  client?.responses?.create?.({ signal });
  agent.responses.create({ signal });
  (client as Client).responses.create({ signal });
  client.responses.create({ signal } satisfies Options);
}

export function ignored(
  client: {
    responses: { create: (options?: object) => void };
    chat: { create: (options?: object) => void; responses: { create: () => void } };
  },
  dynamic: string,
) {
  // Static aliases retain the required option; dynamic shapes stay unclassified.
  const alias = createResponse;
  alias({ signal: null });
  const responses = client.responses;
  responses.create({ signal: null });
  const method = client.responses.create;
  method({ signal: null });
  client[dynamic].create();
  client.responses[dynamic]();
  getClient().responses.create();
  globalThis[dynamic]();
  client.chat.create();
  client.chat.responses.create();
  client.#responses.create();
  globalThis.fetch();
  other();
  for (const { createResponse } of [] as object[]) createResponse();
}

export function shadowedUndefined(undefined: string) {
  createResponse({ signal: undefined });
}

export function unknownValue(signal: string | undefined) {
  let missing: string | undefined;
  createResponse({ signal: missing });
  createResponse({ signal });
}
