import { fetch as importedFetch } from "undici";

declare const client: { fetch(path: string): Promise<unknown> };
declare function createFetch(): (path: string) => Promise<Response>;
declare const name: string;

function withParameter(fetch: (path: string) => Promise<Response>) {
  return fetch("/api/local");
}

const fetch = createFetch();
fetch("/api/local-binding");

importedFetch("/api/imported");
client.fetch("/api/client");

const request = client.fetch;
request("/api/client-alias");

const { fetch: clientFetch } = client;
clientFetch("/api/client-destructure");

const window = { fetch: createFetch() };
window.fetch("/api/window-shadow");

const globalThis = { fetch: createFetch() };
globalThis.fetch("/api/global-shadow");

const dynamic = self[name];
dynamic("/api/dynamic");

// These destructures are intentionally unsupported as fetch aliases.
const { otherFetch, ...restFetchGlobals } = self;
const { fetch: {} } = globalThis;
