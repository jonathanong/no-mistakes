declare function fallback(): void;
declare function local(): void;
declare const client: { responses: { create: (options?: object) => void } };
declare const agent: { responses: { create: (options?: object) => void } };
declare const dynamic: string;

export function missing() {
  const { createResponse } = globalThis;
  const { createResponse: respond } = window;
  const { create: make } = client.responses;
  const { create: typedMake } = (client as Client).responses;
  const { createResponse: defaulted = fallback } = self;
  const { createResponse: fromGlobal, ...ignored } = global;
  createResponse();
  respond();
  make();
  typedMake();
  defaulted();
  fromGlobal();
  createResponse?.();
  make?.();
  const run = () => {
    const { create: inner } = agent.responses;
    inner();
  };
  run();
}

export function present(signal: AbortSignal) {
  const { createResponse } = globalThis;
  const { create: make } = client.responses;
  createResponse({ signal });
  make({ signal });
  make?.({ signal });
}

export function ignored() {
  const alias = createResponse;
  alias();
  const responses = client.responses;
  responses.create();
  const method = client.responses.create;
  method();
  const { [dynamic]: skipped } = globalThis;
  skipped();
  const { create: { nested } = fallback } = client.responses;
  nested();
  let { createResponse: mutated } = globalThis;
  mutated = local;
  mutated();
  var first = local;
  var { createResponse: first } = globalThis;
  first();
  getClient().responses.create();
  client[dynamic].create();
  globalThis[dynamic]();
  const [createResponse] = [local];
  createResponse();
  const { create } = getClient();
  create();
}

declare function getClient(): { responses: { create: (options?: object) => void } };

export function beforeBinding() {
  const run = () => createResponse();
  const { createResponse } = globalThis;
  run();
}
